#![deny(unsafe_code)]

//! Interactive CAD Studio Dialog for Phase 418: Phonon Studio Quantum Metamaterial
//! Topological Acoustic Floquet Spin-Hall Insulator & Non-Reciprocal Cryogenic Circulator.
//!
//! Visualizes acoustic pseudo-spin-1/2 helical edge states with backscattering immunity
//! around sharp corners, spatio-temporal Floquet 3-port non-reciprocal circulation,
//! and cryogenic millikelvin quantum qubit dispersive readout in pure safe Rust.

use egui::{pos2, vec2, Color32, Context, RichText, Sense, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints, VLine};
use phonon_solver::floquet_spinhall_circulator::{
    CirculatorSMatrix, CryogenicReadoutParams, CryogenicReadoutPoint, FloquetCirculatorParams,
    FloquetSpinHallAuditReport, FloquetSpinHallCirculator, FloquetSpinHallParams,
    SpinHallEdgeMode, SpinHallParams, SpinHallPseudoSpin,
};

/// Active tab within the Floquet Spin-Hall Circulator Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloquetSpinHallTab {
    SpinHallWaveguide,
    FloquetCirculator,
    CryogenicReadout,
    RealSpaceMetamaterial,
    AuditTelemetry,
}

impl FloquetSpinHallTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::SpinHallWaveguide => "Spin-Hall Waveguide",
            Self::FloquetCirculator => "Floquet Circulator",
            Self::CryogenicReadout => "Cryogenic Qubit Readout",
            Self::RealSpaceMetamaterial => "2D Real-Space Metamaterial",
            Self::AuditTelemetry => "Physics Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Phase 418.
pub struct FloquetSpinHallCirculatorDialog {
    pub is_open: bool,
    pub active_tab: FloquetSpinHallTab,

    // Spin-Hall Parameters
    pub bare_frequency_ghz: f64,
    pub lattice_constant_a_mm: f64,
    pub inter_intra_ratio: f64,
    pub corner_angle_deg: f64,

    // Floquet Circulator Parameters
    pub pump_freq_mhz: f64,
    pub modulation_depth: f64,
    pub port_coupling_kappa_mhz: f64,
    pub circulation_direction: i32,

    // Cryogenic Readout Parameters
    pub temperature_k: f64,
    pub dispersive_shift_2chi_mhz: f64,
    pub cavity_linewidth_mhz: f64,
    pub probe_photon_count: f64,

    // Real-space display toggle
    pub obstacle_defect_enabled: bool,

    // Solvers & Cached Results
    pub system: FloquetSpinHallCirculator,
    pub cached_dispersion: Vec<SpinHallEdgeMode>,
    pub cached_circulator_spectrum: Vec<CirculatorSMatrix>,
    pub cached_readout_spectrum: Vec<CryogenicReadoutPoint>,
    pub cached_realspace_field: Vec<Vec<f64>>,
    pub cached_audit: FloquetSpinHallAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for FloquetSpinHallCirculatorDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl FloquetSpinHallCirculatorDialog {
    /// Fast cold-boot constructor with pre-seeded baseline data (< 2.0 ms latency).
    pub fn new_fast() -> Self {
        let params = FloquetSpinHallParams::default();
        let system = FloquetSpinHallCirculator::new(params.clone());

        // Pre-seeded lightweight baseline points
        let cached_dispersion = system.lattice.compute_helical_edge_dispersion(25);
        let cached_circulator_spectrum = system.circulator.compute_spectrum(25, 80.0);
        let cached_readout_spectrum = system.readout.compute_readout_spectrum(25, 60.0);
        let cached_realspace_field = system.lattice.compute_realspace_intensity_field(24, 16, false);
        let cached_audit = system.audit_spinhall_circulator();

        Self {
            is_open: false,
            active_tab: FloquetSpinHallTab::SpinHallWaveguide,

            bare_frequency_ghz: params.lattice.bare_frequency_ghz,
            lattice_constant_a_mm: params.lattice.lattice_constant_a_mm,
            inter_intra_ratio: params.lattice.inter_intra_ratio,
            corner_angle_deg: params.lattice.corner_angle_deg,

            pump_freq_mhz: params.circulator.pump_freq_mhz,
            modulation_depth: params.circulator.modulation_depth,
            port_coupling_kappa_mhz: params.circulator.port_coupling_kappa_mhz,
            circulation_direction: params.circulator.circulation_direction,

            temperature_k: params.readout.temperature_k,
            dispersive_shift_2chi_mhz: params.readout.dispersive_shift_2chi_mhz,
            cavity_linewidth_mhz: params.readout.cavity_linewidth_mhz,
            probe_photon_count: params.readout.probe_photon_count,

            obstacle_defect_enabled: false,

            system,
            cached_dispersion,
            cached_circulator_spectrum,
            cached_readout_spectrum,
            cached_realspace_field,
            cached_audit,
            last_solve_time_us: 120.0,
        }
    }

    /// Recomputes all physical spectra and updates the physics audit report.
    pub fn recompute(&mut self) {
        let t0 = crate::time_util::Instant::now();

        let params = FloquetSpinHallParams {
            lattice: SpinHallParams {
                bare_frequency_ghz: self.bare_frequency_ghz,
                lattice_constant_a_mm: self.lattice_constant_a_mm,
                inter_intra_ratio: self.inter_intra_ratio,
                acoustic_velocity_m_s: 3400.0,
                corner_angle_deg: self.corner_angle_deg,
                boundary_cells: 32,
            },
            circulator: FloquetCirculatorParams {
                center_freq_ghz: self.bare_frequency_ghz,
                pump_freq_mhz: self.pump_freq_mhz,
                modulation_depth: self.modulation_depth,
                circulation_direction: self.circulation_direction,
                port_coupling_kappa_mhz: self.port_coupling_kappa_mhz,
                intrinsic_loss_mhz: 0.60,
            },
            readout: CryogenicReadoutParams {
                temperature_k: self.temperature_k,
                readout_freq_ghz: self.bare_frequency_ghz,
                dispersive_shift_2chi_mhz: self.dispersive_shift_2chi_mhz,
                cavity_linewidth_mhz: self.cavity_linewidth_mhz,
                circulator_insertion_loss_db: 0.60,
                circulator_isolation_db: 32.0,
                amplifier_noise_temp_k: 4.0,
                probe_photon_count: self.probe_photon_count,
            },
        };

        self.system = FloquetSpinHallCirculator::new(params);
        self.cached_dispersion = self.system.lattice.compute_helical_edge_dispersion(35);
        self.cached_circulator_spectrum = self.system.circulator.compute_spectrum(45, 80.0);
        self.cached_readout_spectrum = self.system.readout.compute_readout_spectrum(45, 60.0);
        self.cached_realspace_field = self
            .system
            .lattice
            .compute_realspace_intensity_field(28, 18, self.obstacle_defect_enabled);
        self.cached_audit = self.system.audit_spinhall_circulator();

        self.last_solve_time_us = t0.elapsed().as_micros() as f64;
    }

    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Renders the modal window in egui.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new(RichText::new("Topological Acoustic Floquet Spin-Hall Insulator & Cryogenic Circulator").strong())
            .open(&mut is_open)
            .default_size(vec2(880.0, 620.0))
            .min_size(vec2(740.0, 520.0))
            .show(ctx, |ui| {
                self.render_contents(ui);
            });
        self.is_open = is_open;
    }

    /// Renders dialog contents and active tabs.
    pub fn render_contents(&mut self, ui: &mut Ui) {
        // Tab Header
        ui.horizontal(|ui| {
            let tabs = [
                FloquetSpinHallTab::SpinHallWaveguide,
                FloquetSpinHallTab::FloquetCirculator,
                FloquetSpinHallTab::CryogenicReadout,
                FloquetSpinHallTab::RealSpaceMetamaterial,
                FloquetSpinHallTab::AuditTelemetry,
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
            FloquetSpinHallTab::SpinHallWaveguide => self.render_tab_spinhall_waveguide(ui),
            FloquetSpinHallTab::FloquetCirculator => self.render_tab_floquet_circulator(ui),
            FloquetSpinHallTab::CryogenicReadout => self.render_tab_cryogenic_readout(ui),
            FloquetSpinHallTab::RealSpaceMetamaterial => self.render_tab_realspace_metamaterial(ui),
            FloquetSpinHallTab::AuditTelemetry => self.render_tab_audit_telemetry(ui),
        }

        ui.separator();
        self.render_footer(ui);
    }

    fn render_tab_spinhall_waveguide(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("C6v Phononic Crystal Lattice & Helical Edge States").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Apply Topological R/a=1.15 Preset").clicked() {
                    self.inter_intra_ratio = 1.15;
                    self.corner_angle_deg = 60.0;
                    self.recompute();
                }
            });
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let mut changed = false;
            ui.label("R/a Ratio:");
            changed |= ui.add(egui::Slider::new(&mut self.inter_intra_ratio, 0.80..=1.30).step_by(0.01)).changed();

            ui.label("Corner Angle (deg):");
            changed |= ui.add(egui::Slider::new(&mut self.corner_angle_deg, 30.0..=150.0).step_by(5.0)).changed();

            ui.label("Center Freq (GHz):");
            changed |= ui.add(egui::Slider::new(&mut self.bare_frequency_ghz, 0.5..=2.0).step_by(0.1)).changed();

            if changed {
                self.recompute();
            }
        });

        ui.add_space(8.0);

        // Helical edge dispersion plot (Spin-Up in Cyan, Spin-Down in Orange)
        let up_pts: PlotPoints = self
            .cached_dispersion
            .iter()
            .filter(|m| m.pseudo_spin == SpinHallPseudoSpin::SpinUp)
            .map(|m| [m.wavenumber_k, m.frequency_ghz])
            .collect();

        let down_pts: PlotPoints = self
            .cached_dispersion
            .iter()
            .filter(|m| m.pseudo_spin == SpinHallPseudoSpin::SpinDown)
            .map(|m| [m.wavenumber_k, m.frequency_ghz])
            .collect();

        let up_line = Line::new("Spin-Up (+1/2) Forward", up_pts)
            .color(Color32::from_rgb(0, 230, 255))
            .width(2.5);

        let down_line = Line::new("Spin-Down (-1/2) Backward", down_pts)
            .color(Color32::from_rgb(255, 150, 40))
            .width(2.5);

        let f0 = self.bare_frequency_ghz;
        let gap = self.system.lattice.bulk_bandgap_ghz();
        let top_gap = HLine::new("Gap Upper", f0 + gap * 0.5).color(Color32::from_rgb(180, 80, 80));
        let bot_gap = HLine::new("Gap Lower", f0 - gap * 0.5).color(Color32::from_rgb(180, 80, 80));

        Plot::new("plot_spinhall_dispersion")
            .height(250.0)
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(up_line);
                plot_ui.line(down_line);
                plot_ui.hline(top_gap);
                plot_ui.hline(bot_gap);
                plot_ui.vline(VLine::new("k = 0", 0.0).color(Color32::GRAY));
            });
    }

    fn render_tab_floquet_circulator(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Spatio-Temporal Floquet 3-Port Non-Reciprocal Circulator").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Apply 32 dB Isolation Preset").clicked() {
                    self.modulation_depth = 0.28;
                    self.pump_freq_mhz = 50.0;
                    self.recompute();
                }
            });
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let mut changed = false;
            ui.label("Modulation Depth:");
            changed |= ui.add(egui::Slider::new(&mut self.modulation_depth, 0.10..=0.45).step_by(0.01)).changed();

            ui.label("Pump Freq (MHz):");
            changed |= ui.add(egui::Slider::new(&mut self.pump_freq_mhz, 20.0..=100.0).step_by(2.0)).changed();

            ui.label("Coupling (MHz):");
            changed |= ui.add(egui::Slider::new(&mut self.port_coupling_kappa_mhz, 15.0..=50.0).step_by(1.0)).changed();

            if changed {
                self.recompute();
            }
        });

        ui.add_space(8.0);

        let s21_pts: PlotPoints = self
            .cached_circulator_spectrum
            .iter()
            .map(|p| [p.freq_ghz, p.s21_db])
            .collect();
        let s12_pts: PlotPoints = self
            .cached_circulator_spectrum
            .iter()
            .map(|p| [p.freq_ghz, p.s12_db])
            .collect();
        let s11_pts: PlotPoints = self
            .cached_circulator_spectrum
            .iter()
            .map(|p| [p.freq_ghz, p.s11_db])
            .collect();

        let s21_line = Line::new("Forward S21 (dB)", s21_pts)
            .color(Color32::from_rgb(0, 255, 120))
            .width(2.5);
        let s12_line = Line::new("Reverse Isolation S12 (dB)", s12_pts)
            .color(Color32::from_rgb(255, 60, 60))
            .width(2.5);
        let s11_line = Line::new("Input Return Loss S11 (dB)", s11_pts)
            .color(Color32::from_rgb(0, 180, 255))
            .width(2.0);

        Plot::new("plot_floquet_circulator_spectrum")
            .height(260.0)
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(s21_line);
                plot_ui.line(s12_line);
                plot_ui.line(s11_line);
                plot_ui.hline(HLine::new("Target Isolation (-30 dB)", -30.0).color(Color32::from_rgb(200, 100, 100)));
                plot_ui.hline(HLine::new("Return Loss Limit (-22 dB)", -22.0).color(Color32::from_rgb(100, 160, 220)));
            });
    }

    fn render_tab_cryogenic_readout(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Dilution Fridge Cryogenic Qubit Dispersive Readout").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Apply 15 mK Base Preset").clicked() {
                    self.temperature_k = 0.015;
                    self.dispersive_shift_2chi_mhz = 16.0;
                    self.recompute();
                }
            });
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let mut changed = false;
            ui.label("Temp (K):");
            changed |= ui.add(egui::Slider::new(&mut self.temperature_k, 0.010..=0.100).step_by(0.005)).changed();

            ui.label("2*chi (MHz):");
            changed |= ui.add(egui::Slider::new(&mut self.dispersive_shift_2chi_mhz, 8.0..=30.0).step_by(1.0)).changed();

            ui.label("Probe Photons:");
            changed |= ui.add(egui::Slider::new(&mut self.probe_photon_count, 5.0..=40.0).step_by(1.0)).changed();

            if changed {
                self.recompute();
            }
        });

        ui.add_space(8.0);

        let t0_pts: PlotPoints = self
            .cached_readout_spectrum
            .iter()
            .map(|p| [p.freq_ghz, p.trans_state_0_db])
            .collect();
        let t1_pts: PlotPoints = self
            .cached_readout_spectrum
            .iter()
            .map(|p| [p.freq_ghz, p.trans_state_1_db])
            .collect();

        let t0_line = Line::new("Ground State |0>", t0_pts)
            .color(Color32::from_rgb(80, 200, 255))
            .width(2.5);
        let t1_line = Line::new("Excited State |1>", t1_pts)
            .color(Color32::from_rgb(255, 90, 180))
            .width(2.5);

        Plot::new("plot_cryogenic_readout_spectrum")
            .height(260.0)
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(t0_line);
                plot_ui.line(t1_line);
                plot_ui.vline(VLine::new("Cavity f0", self.bare_frequency_ghz).color(Color32::GRAY));
            });
    }

    fn render_tab_realspace_metamaterial(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("2D Real-Space Acoustic Metamaterial & Sharp Corner Routing").strong());
            if ui.checkbox(&mut self.obstacle_defect_enabled, "Simulate Obstacle Defect").changed() {
                self.recompute();
            }
        });

        ui.add_space(8.0);

        let (rect, _response) = ui.allocate_exact_size(vec2(ui.available_width(), 260.0), Sense::hover());
        let painter = ui.painter_at(rect);

        painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 20, 30));

        let rows = self.cached_realspace_field.len();
        let cols = if rows > 0 { self.cached_realspace_field[0].len() } else { 0 };

        if rows > 0 && cols > 0 {
            let dx = rect.width() / (cols as f32);
            let dy = rect.height() / (rows as f32);

            for y in 0..rows {
                for x in 0..cols {
                    let intensity = self.cached_realspace_field[y][x];
                    let col = turbo_color(intensity);
                    let cell_rect = egui::Rect::from_min_size(
                        pos2(rect.left() + (x as f32) * dx, rect.top() + (y as f32) * dy),
                        vec2(dx, dy),
                    );
                    painter.rect_filled(cell_rect, 0.0, col);
                }
            }
        }
    }

    fn render_tab_audit_telemetry(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Physics Audit & Numerical Telemetry Checklist").strong());
        ui.add_space(6.0);

        let audit = &self.cached_audit;

        let checks = [
            ("1. C6v Pseudo-Spin Doublet Degeneracy", audit.pseudo_spin_degeneracy_passed),
            ("2. Quantized Spin Chern Number |C_s| = 1.0", audit.spin_chern_quantized_passed),
            ("3. Gapless Helical Edge State Confinement >= 85%", audit.helical_edge_confinement_passed),
            ("4. Sharp Corner Backscattering Immunity (T >= 99%, S11 <= -22 dB)", audit.corner_backscattering_suppressed),
            ("5. Forward Circulation Insertion Loss IL <= 0.80 dB", audit.forward_insertion_loss_passed),
            ("6. Deep Reverse Isolation ISO >= 30.0 dB", audit.reverse_isolation_passed),
            ("7. Input Return Loss Matched RL <= -22.0 dB", audit.return_loss_matched_passed),
            ("8. Circulation Bandwidth BW >= 25.0 MHz", audit.circulation_bandwidth_passed),
            ("9. Dilution-Fridge Quantum Added Noise n_add <= 0.65 quanta", audit.cryogenic_noise_floor_passed),
            ("10. Qubit Dispersive Readout SNR >= 18.0 dB", audit.readout_snr_passed),
        ];

        for (title, passed) in checks {
            ui.horizontal(|ui| {
                if passed {
                    ui.colored_label(Color32::from_rgb(0, 220, 100), "[PASS]");
                } else {
                    ui.colored_label(Color32::from_rgb(255, 60, 60), "[FAIL]");
                }
                ui.label(title);
            });
        }
    }

    fn render_footer(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(format!("Audit Score: {}/10", self.cached_audit.total_pass_score));
            ui.separator();
            ui.label(format!("Spin Chern: {:.1}", self.system.lattice.spin_chern_number()));
            ui.separator();
            ui.label(format!("Bulk Gap: {:.1} MHz", self.system.lattice.bulk_bandgap_ghz() * 1e3));
            ui.separator();
            ui.label(format!("Isolation: {:.1} dB", self.system.circulator.isolation_contrast_db()));
            ui.separator();
            ui.label(format!("Noise: {:.3} quanta", self.system.readout.quantum_added_noise_quanta()));
            ui.separator();
            ui.label(format!("Solve Latency: {:.1} us", self.last_solve_time_us));
        });
    }
}

/// Simple Turbo colormap for 2D acoustic intensity visualization.
fn turbo_color(val: f64) -> Color32 {
    let v = val.clamp(0.0, 1.0);
    let r = (255.0 * (0.1 + 0.9 * v.powf(0.8))) as u8;
    let g = (255.0 * (0.05 + 0.85 * (1.0 - (v - 0.5).abs() * 2.0).max(0.0))) as u8;
    let b = (255.0 * (0.2 + 0.8 * (1.0 - v).powf(1.5))) as u8;
    Color32::from_rgb(r, g, b)
}
