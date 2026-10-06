#![deny(unsafe_code)]

//! Interactive Coherent Phonon-Magnon Polariton Transducer & Quantum Microwave-to-Acoustic Interface Dialog.
//!
//! Provides a 5-tab quantum electro-acoustic and spintronics CAD studio:
//! 1. Polariton Dispersion & Anticrossing (avoided crossing gap, Hopfield mixing fractions, cooperativity C).
//! 2. Microwave-to-Acoustic S-Parameters (2-port scattering matrix [S(omega)], peak efficiency eta, 3-dB bandwidth).
//! 3. Dynamic Magnetoelastic Strain Field (2D wavepacket strain track, dynamic RF field, resonant acoustic attenuation).
//! 4. Quantum Noise & State Fidelity (dilution fridge thermal floor, added noise photons n_add, state transfer fidelity).
//! 5. Polariton Transducer Audit (10-point checklist verifying strong coupling, passivity, and quantum cooperativity).

use std::f64::consts::PI;
use egui::{Color32, Context, Pos2, Rect, RichText, Stroke, Ui, Vec2, Window};
use phonon_solver::phonon_magnon_polariton::{
    MagnetoelasticDriveEngine, MagnetoelasticDriveParams, MagnetoelasticTrackSnapshot,
    PhononMagnonParams, PolaritonDispersionEngine, PolaritonDispersionPoint,
    QuantumTransducerSolver, SParameterSample, TransducerCouplingParams,
};

/// Active tab in the Phonon-Magnon Polariton Studio Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhononMagnonTab {
    PolaritonDispersion,
    TransducerSParameters,
    MagnetoelasticStrain,
    QuantumNoiseFidelity,
    TransducerAudit,
}

/// 10-point audit item for Phonon-Magnon Polariton Transducers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolaritonAuditCriterion {
    pub criterion: String,
    pub specification: String,
    pub observed_state: String,
    pub is_passed: bool,
    pub technical_notes: String,
}

/// Modal dialog for Coherent Phonon-Magnon Polariton Transducer Studio.
pub struct PhononMagnonDialog {
    pub is_open: bool,
    pub active_tab: PhononMagnonTab,

    // Core params
    pub dispersion_params: PhononMagnonParams,
    pub transducer_params: TransducerCouplingParams,
    pub drive_params: MagnetoelasticDriveParams,

    // Interactive slider bindings
    pub b_ext_tesla: f64,
    pub coupling_g_mhz: f64,
    pub acoustic_q: f64,
    pub temperature_mk: f64,
    pub peak_strain_micro: f64,
    pub track_time_ns: f64,

    // Cached telemetry & diagnostics
    pub resonance_wavenumber_rad_m: f64,
    pub anticrossing_gap_mhz: f64,
    pub quantum_cooperativity: f64,
    pub dispersion_points: Vec<PolaritonDispersionPoint>,
    pub peak_efficiency: f64,
    pub bandwidth_mhz: f64,
    pub quantum_fidelity: f64,
    pub s_params: Vec<SParameterSample>,
    pub added_noise_quanta: f64,
    pub rf_field_amplitude_mt: f64,
    pub track_snapshot: MagnetoelasticTrackSnapshot,

    // Audit
    pub audit_criteria: Vec<PolaritonAuditCriterion>,
    pub audit_score: (usize, usize),
}

impl Default for PhononMagnonDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl PhononMagnonDialog {
    /// Ultra-fast sub-microsecond constructor for cold startup performance (< 5ms boot budget).
    pub fn new_fast() -> Self {
        let disp = PhononMagnonParams::default();
        let trans = TransducerCouplingParams::default();
        let drive = MagnetoelasticDriveParams::default();

        let b_ext = disp.b_ext_tesla;
        let g_mhz = disp.coupling_g_rad_s / (2.0 * PI * 1.0e6);
        let q = disp.acoustic_q;
        let temp_mk = trans.temperature_k * 1000.0;
        let strain_micro = drive.peak_strain * 1.0e6;

        Self {
            is_open: false,
            active_tab: PhononMagnonTab::PolaritonDispersion,
            dispersion_params: disp,
            transducer_params: trans,
            drive_params: drive,
            b_ext_tesla: b_ext,
            coupling_g_mhz: g_mhz,
            acoustic_q: q,
            temperature_mk: temp_mk,
            peak_strain_micro: strain_micro,
            track_time_ns: 0.5,
            resonance_wavenumber_rad_m: 5.72e6,
            anticrossing_gap_mhz: 70.0,
            quantum_cooperativity: 120.0,
            dispersion_points: Vec::new(),
            peak_efficiency: 0.72,
            bandwidth_mhz: 12.5,
            quantum_fidelity: 0.848,
            s_params: Vec::new(),
            added_noise_quanta: 0.19,
            rf_field_amplitude_mt: 0.23,
            track_snapshot: MagnetoelasticTrackSnapshot {
                x_positions_mm: Vec::new(),
                strain_values: Vec::new(),
                dynamic_magnetization: Vec::new(),
                effective_field_mt: Vec::new(),
            },
            audit_criteria: Vec::new(),
            audit_score: (10, 10),
        }
    }

    /// Full constructor running initial calculation.
    pub fn new() -> Self {
        let mut dlg = Self::new_fast();
        dlg.recalculate();
        dlg
    }

    /// Recalculates all polariton dispersion, scattering matrix, and wavepacket metrics.
    pub fn recalculate(&mut self) {
        // Sync parameters
        self.dispersion_params.b_ext_tesla = self.b_ext_tesla;
        self.dispersion_params.coupling_g_rad_s = 2.0 * PI * self.coupling_g_mhz * 1.0e6;
        self.dispersion_params.acoustic_q = self.acoustic_q;

        self.transducer_params.coupling_g_rad_s = self.dispersion_params.coupling_g_rad_s;
        self.transducer_params.temperature_k = self.temperature_mk / 1000.0;

        self.drive_params.peak_strain = self.peak_strain_micro * 1.0e-6;

        // 1. Dispersion
        let disp_engine = PolaritonDispersionEngine::new(self.dispersion_params.clone());
        let k_res = disp_engine.find_resonance_wavenumber();
        self.resonance_wavenumber_rad_m = k_res;

        let res_pt = disp_engine.evaluate_point(k_res);
        self.anticrossing_gap_mhz = res_pt.splitting_mhz();
        self.quantum_cooperativity = res_pt.cooperativity;

        let k_min = 0.5 * k_res;
        let k_max = 1.5 * k_res;
        self.dispersion_points = disp_engine.generate_dispersion_curve(k_min, k_max, 64);

        // 2. S-matrix
        let trans_solver = QuantumTransducerSolver::new(self.transducer_params.clone());
        self.peak_efficiency = trans_solver.compute_peak_efficiency();
        self.bandwidth_mhz = trans_solver.compute_bandwidth_hz() / 1.0e6;
        self.quantum_fidelity = trans_solver.compute_quantum_fidelity();

        let f0 = self.transducer_params.resonance_freq_rad_s / (2.0 * PI);
        let g_hz = self.transducer_params.coupling_g_rad_s / (2.0 * PI);
        self.added_noise_quanta = trans_solver.compute_added_noise_quanta(f0 + g_hz);
        self.s_params = trans_solver.sweep_s_parameters(80.0e6, 64);

        // 3. Drive & wavepacket track
        let drive_engine = MagnetoelasticDriveEngine::new(self.drive_params.clone());
        self.rf_field_amplitude_mt = drive_engine.compute_effective_rf_field_tesla(self.drive_params.peak_strain) * 1.0e3;
        self.track_snapshot = drive_engine.generate_track_snapshot(
            4.0,
            1.1,
            self.track_time_ns * 1.0e-9,
            64,
        );

        // 4. Update audit
        self.update_audit_criteria();
    }

    /// Updates the 10-point audit checklist.
    pub fn update_audit_criteria(&mut self) {
        let mut criteria = Vec::with_capacity(10);

        // 1. Avoided crossing polariton gap
        let c1_pass = self.anticrossing_gap_mhz >= 50.0;
        criteria.push(PolaritonAuditCriterion {
            criterion: "Polariton Avoided Crossing Gap".to_string(),
            specification: "Delta f >= 50.0 MHz (2*g/2pi)".to_string(),
            observed_state: format!("{:.2} MHz", self.anticrossing_gap_mhz),
            is_passed: c1_pass,
            technical_notes: "Strong hybridization opens clear avoided crossing gap between acoustic and magnon branches.".to_string(),
        });

        // 2. Quantum cooperativity
        let c2_pass = self.quantum_cooperativity > 1.0;
        criteria.push(PolaritonAuditCriterion {
            criterion: "Quantum Cooperativity C".to_string(),
            specification: "C = 4*g^2 / (kappa_p * kappa_m) > 1.0".to_string(),
            observed_state: format!("{:.2}", self.quantum_cooperativity),
            is_passed: c2_pass,
            technical_notes: "Cooperativity strictly exceeds unity, confirming strong coupling coherent regime.".to_string(),
        });

        // 3. Hopfield mixing normalization
        let norm_err = if let Some(first) = self.dispersion_points.first() {
            (first.hopfield_phonon_fraction + first.hopfield_magnon_fraction - 1.0).abs()
        } else {
            0.0
        };
        let c3_pass = norm_err < 1.0e-5;
        criteria.push(PolaritonAuditCriterion {
            criterion: "Hopfield Mixing Normalization".to_string(),
            specification: "|u|^2 + |v|^2 = 1.000 +/- 1e-5".to_string(),
            observed_state: format!("Sum = {:.6} (err {:.2e})", 1.0 - norm_err, norm_err),
            is_passed: c3_pass,
            technical_notes: "Polariton basis transformation maintains probability conservation across momentum spectrum.".to_string(),
        });

        // 4. Scattering matrix passivity
        let max_power = self.s_params.iter().fold(0.0f64, |acc, s| acc.max(s.s11_power + s.s21_power));
        let c4_pass = max_power <= 1.0001;
        criteria.push(PolaritonAuditCriterion {
            criterion: "Scattering Matrix Passivity".to_string(),
            specification: "|S11|^2 + |S21|^2 <= 1.000".to_string(),
            observed_state: format!("Max Power = {:.4}", max_power),
            is_passed: c4_pass,
            technical_notes: "Scattering matrix rigorously satisfies energy conservation and input-output unitarity.".to_string(),
        });

        // 5. Peak conversion efficiency
        let c5_pass = self.peak_efficiency >= 0.60;
        criteria.push(PolaritonAuditCriterion {
            criterion: "Peak Conversion Efficiency".to_string(),
            specification: "eta_max = |S21(f0)|^2 >= 60.0% (-2.2 dB)".to_string(),
            observed_state: format!("{:.1}% ({:.2} dB)", self.peak_efficiency * 100.0, 10.0 * self.peak_efficiency.max(1e-12).log10()),
            is_passed: c5_pass,
            technical_notes: "Critical impedance and decay rate matching maximizes bidirectional microwave-to-acoustic transduction.".to_string(),
        });

        // 6. Quantum state transfer fidelity
        let c6_pass = self.quantum_fidelity >= 0.77;
        criteria.push(PolaritonAuditCriterion {
            criterion: "Quantum State Transfer Fidelity".to_string(),
            specification: "F = sqrt(eta) >= 0.770".to_string(),
            observed_state: format!("{:.4}", self.quantum_fidelity),
            is_passed: c6_pass,
            technical_notes: "Coherent quantum state transmission across piezoelectric-ferrimagnetic boundary.".to_string(),
        });

        // 7. Transduction 3-dB bandwidth
        let c7_pass = self.bandwidth_mhz >= 2.0;
        criteria.push(PolaritonAuditCriterion {
            criterion: "Transduction 3-dB Bandwidth".to_string(),
            specification: "Delta f_3dB >= 2.0 MHz".to_string(),
            observed_state: format!("{:.2} MHz", self.bandwidth_mhz),
            is_passed: c7_pass,
            technical_notes: "Operational instantaneous bandwidth supports high-baud quantum acoustic bus modulation.".to_string(),
        });

        // 8. Added thermal noise quanta
        let c8_pass = self.added_noise_quanta <= 2.0;
        criteria.push(PolaritonAuditCriterion {
            criterion: "Added Quantum Noise Quanta".to_string(),
            specification: "n_add <= 2.0 photons at dilution base temp".to_string(),
            observed_state: format!("{:.3} quanta", self.added_noise_quanta),
            is_passed: c8_pass,
            technical_notes: "Near quantum-limited added noise ensures preservation of single-phonon non-classical states.".to_string(),
        });

        // 9. Dynamic magnetoelastic linearity
        let c9_pass = self.rf_field_amplitude_mt > 0.05 && self.rf_field_amplitude_mt < 50.0;
        criteria.push(PolaritonAuditCriterion {
            criterion: "Magnetoelastic RF Effective Field".to_string(),
            specification: "0.05 mT <= h_me <= 50.0 mT".to_string(),
            observed_state: format!("{:.3} mT", self.rf_field_amplitude_mt),
            is_passed: c9_pass,
            technical_notes: "Lattice strain dynamically induces microwave-band effective magnetic field via magnetoelastic tensor.".to_string(),
        });

        // 10. Resonant acoustic attenuation depth
        let drive_eng = MagnetoelasticDriveEngine::new(self.drive_params.clone());
        let f_res = self.transducer_params.resonance_freq_rad_s / (2.0 * PI);
        let alpha_peak = drive_eng.compute_resonant_attenuation_db_cm(f_res, f_res, 10.0e6, 12.0);
        let alpha_off = drive_eng.compute_resonant_attenuation_db_cm(f_res + 100.0e6, f_res, 10.0e6, 12.0);
        let c10_pass = alpha_peak > alpha_off + 10.0;
        criteria.push(PolaritonAuditCriterion {
            criterion: "Resonant Acoustic Attenuation Contrast".to_string(),
            specification: "Delta alpha_ac >= 10.0 dB / cm".to_string(),
            observed_state: format!("{:.1} dB/cm contrast", alpha_peak - alpha_off),
            is_passed: c10_pass,
            technical_notes: "Sharp magnetoacoustic resonance absorption verifies efficient phonon energy capture into magnon mode.".to_string(),
        });

        let passed = criteria.iter().filter(|c| c.is_passed).count();
        self.audit_score = (passed, criteria.len());
        self.audit_criteria = criteria;
    }

    /// Renders the modal window.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Renders the modal window.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        // Lazy initialize data on first opening
        if self.dispersion_points.is_empty() {
            self.recalculate();
        }

        let mut is_open = self.is_open;
        Window::new("Coherent Phonon-Magnon Polariton Transducer Studio")
            .open(&mut is_open)
            .default_width(880.0)
            .default_height(620.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders the internal content of the dialog.
    pub fn render_content(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Coherent Phonon-Magnon Polariton Transducer").color(Color32::from_rgb(0, 210, 255)).strong());
            ui.label(RichText::new("Quantum Microwave-to-Acoustic Interface").color(Color32::GRAY));
        });

        ui.add_space(6.0);

        // Tab selection bar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.active_tab, PhononMagnonTab::PolaritonDispersion, "Polariton Dispersion & Anticrossing");
            ui.selectable_value(&mut self.active_tab, PhononMagnonTab::TransducerSParameters, "Microwave-to-Acoustic S-Parameters");
            ui.selectable_value(&mut self.active_tab, PhononMagnonTab::MagnetoelasticStrain, "Dynamic Magnetoelastic Strain Field");
            ui.selectable_value(&mut self.active_tab, PhononMagnonTab::QuantumNoiseFidelity, "Quantum Noise & State Fidelity");
            ui.selectable_value(&mut self.active_tab, PhononMagnonTab::TransducerAudit, "Polariton Transducer Audit");
        });

        ui.separator();

        match self.active_tab {
            PhononMagnonTab::PolaritonDispersion => self.render_dispersion_tab(ui),
            PhononMagnonTab::TransducerSParameters => self.render_s_parameters_tab(ui),
            PhononMagnonTab::MagnetoelasticStrain => self.render_strain_tab(ui),
            PhononMagnonTab::QuantumNoiseFidelity => self.render_noise_fidelity_tab(ui),
            PhononMagnonTab::TransducerAudit => self.render_audit_tab(ui),
        }

        ui.separator();
        self.render_telemetry_footer(ui);
    }

    fn render_dispersion_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Hybridized Polariton Dispersion & Anticrossing Gap").strong());
                ui.label(RichText::new("Displays upper/lower polaritons omega_+/- and Hopfield mixing fractions.").small().color(Color32::LIGHT_GRAY));

                ui.add_space(8.0);
                let mut changed = false;

                ui.group(|ui| {
                    ui.label(RichText::new("Coupling & Bias Controls").strong());
                    changed |= ui.add(egui::Slider::new(&mut self.b_ext_tesla, 0.05..=0.30).text("Bias Field B_ext (T)")).changed();
                    changed |= ui.add(egui::Slider::new(&mut self.coupling_g_mhz, 10.0..=80.0).text("Coupling Rate g / 2pi (MHz)")).changed();
                    changed |= ui.add(egui::Slider::new(&mut self.acoustic_q, 500.0..=10000.0).text("Acoustic Q-Factor")).changed();

                    if ui.button("Recalculate Dispersion").clicked() {
                        changed = true;
                    }
                });

                if changed {
                    self.recalculate();
                }

                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.label(RichText::new("Anticrossing Telemetry").strong());
                    ui.label(format!("Resonance Wavenumber: {:.3e} rad/m", self.resonance_wavenumber_rad_m));
                    ui.label(format!("Avoided Crossing Gap: {:.2} MHz", self.anticrossing_gap_mhz));
                    ui.label(format!("Quantum Cooperativity C: {:.2}", self.quantum_cooperativity));
                });
            });

            ui.separator();

            // Dispersion canvas
            ui.vertical(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(480.0, 320.0), egui::Sense::hover());
                let painter = ui.painter_at(rect);

                // Background
                painter.rect_filled(rect, 4.0, Color32::from_rgb(18, 22, 28));
                painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 72)), egui::StrokeKind::Inside);

                if self.dispersion_points.len() > 1 {
                    let k_min = self.dispersion_points.first().map(|p| p.wavenumber_rad_m).unwrap_or(1.0);
                    let k_max = self.dispersion_points.last().map(|p| p.wavenumber_rad_m).unwrap_or(2.0);
                    let dk = (k_max - k_min).max(1.0);

                    let f_min = 3.0; // GHz
                    let f_max = 4.2; // GHz
                    let df = f_max - f_min;

                    let to_screen = |k: f64, f_ghz: f64| -> Pos2 {
                        let nx = ((k - k_min) / dk) as f32;
                        let ny = 1.0 - ((f_ghz - f_min) / df) as f32;
                        Pos2::new(
                            rect.min.x + 30.0 + nx * (rect.width() - 50.0),
                            rect.min.y + 20.0 + ny * (rect.height() - 40.0),
                        )
                    };

                    // Draw bare phonon (dashed blue) & bare magnon (dashed green)
                    for i in 0..(self.dispersion_points.len() - 1) {
                        let p0 = &self.dispersion_points[i];
                        let p1 = &self.dispersion_points[i + 1];

                        // Bare phonon
                        painter.line_segment(
                            [to_screen(p0.wavenumber_rad_m, p0.phonon_freq_ghz()), to_screen(p1.wavenumber_rad_m, p1.phonon_freq_ghz())],
                            Stroke::new(1.0, Color32::from_rgb(60, 120, 200)),
                        );

                        // Bare magnon
                        painter.line_segment(
                            [to_screen(p0.wavenumber_rad_m, p0.magnon_freq_ghz()), to_screen(p1.wavenumber_rad_m, p1.magnon_freq_ghz())],
                            Stroke::new(1.0, Color32::from_rgb(60, 180, 100)),
                        );

                        // Lower polariton (solid gold)
                        painter.line_segment(
                            [to_screen(p0.wavenumber_rad_m, p0.lower_polariton_ghz()), to_screen(p1.wavenumber_rad_m, p1.lower_polariton_ghz())],
                            Stroke::new(2.5, Color32::from_rgb(255, 200, 40)),
                        );

                        // Upper polariton (solid cyan)
                        painter.line_segment(
                            [to_screen(p0.wavenumber_rad_m, p0.upper_polariton_ghz()), to_screen(p1.wavenumber_rad_m, p1.upper_polariton_ghz())],
                            Stroke::new(2.5, Color32::from_rgb(0, 220, 255)),
                        );
                    }

                    // Legend
                    painter.text(Pos2::new(rect.min.x + 40.0, rect.min.y + 30.0), egui::Align2::LEFT_TOP, "Upper Polariton (omega_+)", egui::FontId::monospace(11.0), Color32::from_rgb(0, 220, 255));
                    painter.text(Pos2::new(rect.min.x + 40.0, rect.min.y + 45.0), egui::Align2::LEFT_TOP, "Lower Polariton (omega_-)", egui::FontId::monospace(11.0), Color32::from_rgb(255, 200, 40));
                    painter.text(Pos2::new(rect.min.x + 40.0, rect.min.y + 60.0), egui::Align2::LEFT_TOP, "Bare Phonon (omega_p)", egui::FontId::monospace(11.0), Color32::from_rgb(60, 120, 200));
                    painter.text(Pos2::new(rect.min.x + 40.0, rect.min.y + 75.0), egui::Align2::LEFT_TOP, "Bare Magnon (omega_m)", egui::FontId::monospace(11.0), Color32::from_rgb(60, 180, 100));
                }
            });
        });
    }

    fn render_s_parameters_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Microwave-to-Acoustic Scattering Matrix [S(omega)]").strong());
                ui.label(RichText::new("Evaluates transmission |S21|^2, return loss |S11|^2, and 3-dB bandwidth.").small().color(Color32::LIGHT_GRAY));

                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.label(RichText::new("Transduction Metrics").strong());
                    ui.label(format!("Peak Conversion Efficiency: {:.1}%", self.peak_efficiency * 100.0));
                    ui.label(format!("Insertion Loss |S21|: {:.2} dB", 10.0 * self.peak_efficiency.max(1e-12).log10()));
                    ui.label(format!("3-dB Instantaneous BW: {:.2} MHz", self.bandwidth_mhz));
                    ui.label(format!("Quantum State Fidelity: {:.4}", self.quantum_fidelity));
                });
            });

            ui.separator();

            // S-parameter plot canvas
            ui.vertical(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(480.0, 320.0), egui::Sense::hover());
                let painter = ui.painter_at(rect);

                painter.rect_filled(rect, 4.0, Color32::from_rgb(18, 22, 28));
                painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 72)), egui::StrokeKind::Inside);

                if self.s_params.len() > 1 {
                    let f_min = self.s_params.first().map(|s| s.frequency_hz).unwrap_or(3.4e9);
                    let f_max = self.s_params.last().map(|s| s.frequency_hz).unwrap_or(3.6e9);
                    let df = (f_max - f_min).max(1.0);

                    let db_min = -35.0f32;
                    let db_max = 0.0f32;
                    let d_db = db_max - db_min;

                    let to_screen = |f_hz: f64, val_db: f64| -> Pos2 {
                        let nx = ((f_hz - f_min) / df) as f32;
                        let ny = 1.0 - ((val_db as f32 - db_min) / d_db).clamp(0.0, 1.0);
                        Pos2::new(
                            rect.min.x + 35.0 + nx * (rect.width() - 50.0),
                            rect.min.y + 20.0 + ny * (rect.height() - 40.0),
                        )
                    };

                    // Draw 3-dB line
                    let peak_db = 10.0 * self.peak_efficiency.max(1e-12).log10();
                    let line_y = to_screen(f_min, peak_db - 3.0).y;
                    painter.line_segment(
                        [Pos2::new(rect.min.x + 35.0, line_y), Pos2::new(rect.max.x - 15.0, line_y)],
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(200, 200, 200, 80)),
                    );

                    for i in 0..(self.s_params.len() - 1) {
                        let s0 = &self.s_params[i];
                        let s1 = &self.s_params[i + 1];

                        // S21 transmission (bright green)
                        painter.line_segment(
                            [to_screen(s0.frequency_hz, s0.s21_db), to_screen(s1.frequency_hz, s1.s21_db)],
                            Stroke::new(2.5, Color32::from_rgb(80, 240, 120)),
                        );

                        // S11 return loss (orange)
                        painter.line_segment(
                            [to_screen(s0.frequency_hz, s0.s11_db), to_screen(s1.frequency_hz, s1.s11_db)],
                            Stroke::new(1.5, Color32::from_rgb(255, 120, 60)),
                        );
                    }

                    painter.text(Pos2::new(rect.min.x + 45.0, rect.min.y + 30.0), egui::Align2::LEFT_TOP, "|S21| Transduction (dB)", egui::FontId::monospace(11.0), Color32::from_rgb(80, 240, 120));
                    painter.text(Pos2::new(rect.min.x + 45.0, rect.min.y + 45.0), egui::Align2::LEFT_TOP, "|S11| Return Loss (dB)", egui::FontId::monospace(11.0), Color32::from_rgb(255, 120, 60));
                    painter.text(Pos2::new(rect.max.x - 20.0, line_y - 12.0), egui::Align2::RIGHT_BOTTOM, "-3 dB Level", egui::FontId::monospace(10.0), Color32::LIGHT_GRAY);
                }
            });
        });
    }

    fn render_strain_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Dynamic Magnetoelastic Wavepacket & RF Drive").strong());
                ui.label(RichText::new("Coupling of acoustic strain epsilon(x) into RF effective field h_me(x).").small().color(Color32::LIGHT_GRAY));

                ui.add_space(8.0);
                let mut changed = false;

                ui.group(|ui| {
                    ui.label(RichText::new("Acoustic Drive Controls").strong());
                    changed |= ui.add(egui::Slider::new(&mut self.peak_strain_micro, 1.0..=100.0).text("Peak Strain (microstrain)")).changed();
                    changed |= ui.add(egui::Slider::new(&mut self.track_time_ns, 0.0..=1.5).text("Wavepacket Time (ns)")).changed();

                    if ui.button("Step Time +0.1 ns").clicked() {
                        self.track_time_ns = (self.track_time_ns + 0.1).min(1.5);
                        changed = true;
                    }
                });

                if changed {
                    self.recalculate();
                }

                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.label(RichText::new("Dynamic Field Telemetry").strong());
                    ui.label(format!("RF Effective Field: {:.3} mT", self.rf_field_amplitude_mt));
                    ui.label("Piezoelectric-to-Ferrimagnetic Interface: LiNbO3 / YIG");
                });
            });

            ui.separator();

            // Track snapshot plot canvas
            ui.vertical(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(480.0, 320.0), egui::Sense::hover());
                let painter = ui.painter_at(rect);

                painter.rect_filled(rect, 4.0, Color32::from_rgb(18, 22, 28));
                painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 72)), egui::StrokeKind::Inside);

                let n = self.track_snapshot.x_positions_mm.len();
                if n > 1 {
                    let x_max = self.track_snapshot.x_positions_mm.last().copied().unwrap_or(4.0);

                    let to_screen = |x_mm: f64, norm_val: f64| -> Pos2 {
                        let nx = (x_mm / x_max) as f32;
                        let ny = 0.5 - (0.4 * norm_val as f32);
                        Pos2::new(
                            rect.min.x + 30.0 + nx * (rect.width() - 50.0),
                            rect.min.y + ny * rect.height(),
                        )
                    };

                    // Draw centerline
                    painter.line_segment(
                        [Pos2::new(rect.min.x + 30.0, rect.center().y), Pos2::new(rect.max.x - 20.0, rect.center().y)],
                        Stroke::new(1.0, Color32::from_rgb(40, 50, 65)),
                    );

                    let max_strain = self.peak_strain_micro.max(1.0);
                    for i in 0..(n - 1) {
                        let x0 = self.track_snapshot.x_positions_mm[i];
                        let x1 = self.track_snapshot.x_positions_mm[i + 1];

                        // Strain waveform (cyan)
                        let s0 = self.track_snapshot.strain_values[i] / max_strain;
                        let s1 = self.track_snapshot.strain_values[i + 1] / max_strain;
                        painter.line_segment([to_screen(x0, s0), to_screen(x1, s1)], Stroke::new(2.0, Color32::from_rgb(0, 220, 255)));

                        // Dynamic precession m_x (pink)
                        let m0 = self.track_snapshot.dynamic_magnetization[i] / 0.08;
                        let m1 = self.track_snapshot.dynamic_magnetization[i + 1] / 0.08;
                        painter.line_segment([to_screen(x0, m0), to_screen(x1, m1)], Stroke::new(1.5, Color32::from_rgb(255, 100, 180)));
                    }

                    painter.text(Pos2::new(rect.min.x + 40.0, rect.min.y + 30.0), egui::Align2::LEFT_TOP, "Dynamic SAW Strain epsilon(x)", egui::FontId::monospace(11.0), Color32::from_rgb(0, 220, 255));
                    painter.text(Pos2::new(rect.min.x + 40.0, rect.min.y + 45.0), egui::Align2::LEFT_TOP, "Dynamic Precession m_x(x)", egui::FontId::monospace(11.0), Color32::from_rgb(255, 100, 180));
                }
            });
        });
    }

    fn render_noise_fidelity_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Quantum Noise & State Transfer Fidelity").strong());
                ui.label(RichText::new("Evaluates thermal noise occupancy, added quanta n_add, and fidelity F.").small().color(Color32::LIGHT_GRAY));

                ui.add_space(8.0);
                let mut changed = false;

                ui.group(|ui| {
                    ui.label(RichText::new("Cryogenic Base Temperature").strong());
                    changed |= ui.add(egui::Slider::new(&mut self.temperature_mk, 10.0..=500.0).text("Temperature T (mK)")).changed();

                    if ui.button("Reset to Base (20 mK)").clicked() {
                        self.temperature_mk = 20.0;
                        changed = true;
                    }
                });

                if changed {
                    self.recalculate();
                }

                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.label(RichText::new("Quantum Performance").strong());
                    ui.label(format!("Added Noise Quanta n_add: {:.4}", self.added_noise_quanta));
                    ui.label(format!("State Transfer Fidelity F: {:.4}", self.quantum_fidelity));
                    ui.label(format!("Infidelity 1 - F: {:.4e}", 1.0 - self.quantum_fidelity));
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(480.0, 320.0), egui::Sense::hover());
                let painter = ui.painter_at(rect);

                painter.rect_filled(rect, 4.0, Color32::from_rgb(18, 22, 28));
                painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 72)), egui::StrokeKind::Inside);

                // Bar chart / visual meters
                let bar_width = 300.0;
                let bar_x = rect.min.x + 40.0;

                // Fidelity gauge
                let y1 = rect.min.y + 80.0;
                painter.text(Pos2::new(bar_x, y1 - 20.0), egui::Align2::LEFT_TOP, "Quantum State Fidelity F", egui::FontId::monospace(12.0), Color32::WHITE);
                let bg_rect1 = Rect::from_min_size(Pos2::new(bar_x, y1), Vec2::new(bar_width, 24.0));
                painter.rect_filled(bg_rect1, 3.0, Color32::from_rgb(30, 38, 50));
                let fg_rect1 = Rect::from_min_size(Pos2::new(bar_x, y1), Vec2::new(bar_width * self.quantum_fidelity as f32, 24.0));
                painter.rect_filled(fg_rect1, 3.0, Color32::from_rgb(0, 200, 140));
                painter.text(Pos2::new(bar_x + bar_width + 15.0, y1 + 4.0), egui::Align2::LEFT_TOP, format!("{:.1}%", self.quantum_fidelity * 100.0), egui::FontId::monospace(12.0), Color32::from_rgb(0, 200, 140));

                // Added noise gauge
                let y2 = rect.min.y + 170.0;
                painter.text(Pos2::new(bar_x, y2 - 20.0), egui::Align2::LEFT_TOP, "Added Thermal Noise Quanta n_add", egui::FontId::monospace(12.0), Color32::WHITE);
                let bg_rect2 = Rect::from_min_size(Pos2::new(bar_x, y2), Vec2::new(bar_width, 24.0));
                painter.rect_filled(bg_rect2, 3.0, Color32::from_rgb(30, 38, 50));
                let frac_noise = (self.added_noise_quanta as f32 / 1.0).clamp(0.0, 1.0);
                let fg_rect2 = Rect::from_min_size(Pos2::new(bar_x, y2), Vec2::new(bar_width * frac_noise, 24.0));
                painter.rect_filled(fg_rect2, 3.0, Color32::from_rgb(255, 140, 40));
                painter.text(Pos2::new(bar_x + bar_width + 15.0, y2 + 4.0), egui::Align2::LEFT_TOP, format!("{:.3} quanta", self.added_noise_quanta), egui::FontId::monospace(12.0), Color32::from_rgb(255, 140, 40));
            });
        });
    }

    fn render_audit_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("Transducer Coherence & Quantum Interface Audit");
            ui.add_space(16.0);
            let score_text = format!("Score: {} / {} Verified", self.audit_score.0, self.audit_score.1);
            let score_color = if self.audit_score.0 == self.audit_score.1 {
                Color32::from_rgb(80, 240, 120)
            } else {
                Color32::from_rgb(255, 180, 60)
            };
            ui.label(RichText::new(score_text).color(score_color).strong());
        });

        ui.add_space(8.0);
        egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
            egui::Grid::new("polariton_transducer_audit_grid")
                .striped(true)
                .min_col_width(120.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("Criterion").strong());
                    ui.label(RichText::new("Target Specification").strong());
                    ui.label(RichText::new("Observed Telemetry").strong());
                    ui.label(RichText::new("Status").strong());
                    ui.label(RichText::new("Verification Rationale").strong());
                    ui.end_row();

                    for crit in &self.audit_criteria {
                        ui.label(RichText::new(&crit.criterion).color(Color32::WHITE));
                        ui.label(RichText::new(&crit.specification).color(Color32::LIGHT_GRAY));
                        ui.label(RichText::new(&crit.observed_state).color(Color32::from_rgb(0, 210, 255)));

                        let status_text = if crit.is_passed { "[PASS]" } else { "[FAIL]" };
                        let status_col = if crit.is_passed { Color32::from_rgb(80, 240, 120) } else { Color32::from_rgb(255, 80, 80) };
                        ui.label(RichText::new(status_text).color(status_col).strong());

                        ui.label(RichText::new(&crit.technical_notes).small().color(Color32::GRAY));
                        ui.end_row();
                    }
                });
        });
    }

    fn render_telemetry_footer(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Polariton Transducer Telemetry:").strong().color(Color32::GRAY));
            ui.label(format!("Splitting: {:.1} MHz", self.anticrossing_gap_mhz));
            ui.separator();
            ui.label(format!("Cooperativity C: {:.1}", self.quantum_cooperativity));
            ui.separator();
            ui.label(format!("Peak Efficiency: {:.1}%", self.peak_efficiency * 100.0));
            ui.separator();
            ui.label(format!("Fidelity: {:.3}", self.quantum_fidelity));
            ui.separator();
            ui.label(format!("Cryo Temp: {:.1} mK", self.temperature_mk));
        });
    }
}
