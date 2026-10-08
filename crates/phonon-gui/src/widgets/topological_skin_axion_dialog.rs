#![deny(unsafe_code)]

//! Phase 446: Topological Phononic Non-Hermitian Skin-Effect Microwave Amplification & Directional Axion Transducer Dialog.
//!
//! Interactive 5-tab CAD modal dialog for exploring non-Hermitian skin-effect (NHSE)
//! traveling-wave microwave amplification, generalized Brillouin zone point-gap topology,
//! coherent dark-matter axion-phonon transduction, and cryogenic readout crossbars.

use egui::{Color32, Context, RichText, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::topological_skin_axion::{
    AxionCouplingScanPoint, AxionPhononTransducerSolver, AxionResonancePoint,
    AxionTransducerMetrics, AxionTransducerParams, CrossbarLinearityPoint,
    CrossbarSParameterPoint, CryogenicCrossbarMetrics, CryogenicCrossbarParams,
    CryogenicReadoutCrossbarSolver, GainBandwidthPoint, GbzPoint, SkinAmplifierMetrics,
    SkinAmplifierParams, SkinAxionAuditReport, SkinMicrowaveAmplifierSolver,
    SkinSpatialProfilePoint, TopologicalSkinAxionProcessor,
};

/// 5 Categorized navigation tabs for the topological skin-axion dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkinAxionDialogTab {
    SkinAmplifierGbz,
    SpatialSkinModes,
    AxionPhononConversion,
    CryogenicReadoutCrossbar,
    AuditTelemetry,
}

impl SkinAxionDialogTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::SkinAmplifierGbz => "Skin Amplifier & GBZ",
            Self::SpatialSkinModes => "Spatial Skin Modes",
            Self::AxionPhononConversion => "Axion-Phonon Conversion",
            Self::CryogenicReadoutCrossbar => "Cryogenic Crossbar",
            Self::AuditTelemetry => "Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Topological Skin-Effect Axion Transducer (Phase 446).
#[derive(Debug, Clone)]
pub struct TopologicalSkinAxionDialog {
    pub is_open: bool,
    pub active_tab: SkinAxionDialogTab,

    // Amplifier parameters
    pub forward_hopping_tr_mhz: f64,
    pub reverse_hopping_tl_mhz: f64,
    pub lattice_site_count: usize,

    // Axion transducer parameters
    pub magnetic_field_t: f64,
    pub axion_mass_micro_ev: f64,
    pub acoustic_quality_factor_qm: f64,

    // Crossbar parameters
    pub operating_temp_k: f64,
    pub pump_frequency_ghz: f64,
    pub dispersive_shift_chi_mhz: f64,
    pub cavity_linewidth_kappa_mhz: f64,

    // Cached models and telemetry
    pub cached_amp_metrics: SkinAmplifierMetrics,
    pub cached_gbz_spectrum: Vec<GbzPoint>,
    pub cached_spatial_modes: Vec<SkinSpatialProfilePoint>,
    pub cached_gain_bandwidth: Vec<GainBandwidthPoint>,

    pub cached_axion_metrics: AxionTransducerMetrics,
    pub cached_coupling_scan: Vec<AxionCouplingScanPoint>,
    pub cached_resonance_curve: Vec<AxionResonancePoint>,

    pub cached_crossbar_metrics: CryogenicCrossbarMetrics,
    pub cached_s_parameters: Vec<CrossbarSParameterPoint>,
    pub cached_linearity: Vec<CrossbarLinearityPoint>,

    pub cached_audit: SkinAxionAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for TopologicalSkinAxionDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl TopologicalSkinAxionDialog {
    /// Instantaneous cold-boot constructor (< 2ms) with pre-seeded baseline state.
    pub fn new_fast() -> Self {
        let amp_params = SkinAmplifierParams::default();
        let axion_params = AxionTransducerParams::default();
        let cross_params = CryogenicCrossbarParams::default();

        let amp_solver = SkinMicrowaveAmplifierSolver::new(amp_params.clone());
        let axion_solver = AxionPhononTransducerSolver::new(axion_params.clone());
        let cross_solver = CryogenicReadoutCrossbarSolver::new(cross_params.clone());

        let cached_amp_metrics = amp_solver.evaluate_metrics();
        let cached_gbz_spectrum = amp_solver.compute_gbz_spectrum(32);
        let cached_spatial_modes = amp_solver.compute_spatial_skin_modes();
        let cached_gain_bandwidth = amp_solver.compute_gain_bandwidth_spectrum(32);

        let cached_axion_metrics = axion_solver.evaluate_metrics();
        let cached_coupling_scan = axion_solver.compute_coupling_scan(32);
        let cached_resonance_curve = axion_solver.compute_conversion_resonance_curve(32);

        let cached_crossbar_metrics = cross_solver.evaluate_metrics();
        let cached_s_parameters = cross_solver.compute_s_parameters(32);
        let cached_linearity = cross_solver.compute_dynamic_range_linearity(32);

        let processor = TopologicalSkinAxionProcessor::new(
            amp_params.clone(),
            axion_params.clone(),
            cross_params.clone(),
        );
        let cached_audit = processor.audit_system();

        Self {
            is_open: false,
            active_tab: SkinAxionDialogTab::SkinAmplifierGbz,

            forward_hopping_tr_mhz: amp_params.forward_hopping_tr_mhz,
            reverse_hopping_tl_mhz: amp_params.reverse_hopping_tl_mhz,
            lattice_site_count: amp_params.lattice_site_count,

            magnetic_field_t: axion_params.magnetic_field_t,
            axion_mass_micro_ev: axion_params.axion_mass_micro_ev,
            acoustic_quality_factor_qm: axion_params.acoustic_quality_factor_qm,

            operating_temp_k: cross_params.operating_temp_k,
            pump_frequency_ghz: cross_params.pump_frequency_ghz,
            dispersive_shift_chi_mhz: cross_params.dispersive_shift_chi_mhz,
            cavity_linewidth_kappa_mhz: cross_params.cavity_linewidth_kappa_mhz,

            cached_amp_metrics,
            cached_gbz_spectrum,
            cached_spatial_modes,
            cached_gain_bandwidth,

            cached_axion_metrics,
            cached_coupling_scan,
            cached_resonance_curve,

            cached_crossbar_metrics,
            cached_s_parameters,
            cached_linearity,

            cached_audit,
            last_solve_time_us: 110.0,
        }
    }

    /// Recomputes all physics solvers with current parameter settings.
    pub fn recompute(&mut self) {
        let start = crate::time_util::Instant::now();

        let amp_params = SkinAmplifierParams {
            forward_hopping_tr_mhz: self.forward_hopping_tr_mhz,
            reverse_hopping_tl_mhz: self.reverse_hopping_tl_mhz,
            onsite_detuning_mhz: 0.0,
            lattice_site_count: self.lattice_site_count,
            acoustic_velocity_ms: 3450.0,
            operating_temp_k: self.operating_temp_k,
        };

        let axion_params = AxionTransducerParams {
            magnetic_field_t: self.magnetic_field_t,
            axion_mass_micro_ev: self.axion_mass_micro_ev,
            acoustic_quality_factor_qm: self.acoustic_quality_factor_qm,
            cavity_volume_cm3: 15.0,
            magnetoelastic_coupling_ge: 1.8e-4,
            integration_time_s: 1.0,
        };

        let cross_params = CryogenicCrossbarParams {
            port_count: 4,
            operating_temp_k: self.operating_temp_k,
            pump_frequency_ghz: self.pump_frequency_ghz,
            probe_power_dbm: -110.0,
            dispersive_shift_chi_mhz: self.dispersive_shift_chi_mhz,
            cavity_linewidth_kappa_mhz: self.cavity_linewidth_kappa_mhz,
        };

        let amp_solver = SkinMicrowaveAmplifierSolver::new(amp_params.clone());
        let axion_solver = AxionPhononTransducerSolver::new(axion_params.clone());
        let cross_solver = CryogenicReadoutCrossbarSolver::new(cross_params.clone());

        self.cached_amp_metrics = amp_solver.evaluate_metrics();
        self.cached_gbz_spectrum = amp_solver.compute_gbz_spectrum(32);
        self.cached_spatial_modes = amp_solver.compute_spatial_skin_modes();
        self.cached_gain_bandwidth = amp_solver.compute_gain_bandwidth_spectrum(32);

        self.cached_axion_metrics = axion_solver.evaluate_metrics();
        self.cached_coupling_scan = axion_solver.compute_coupling_scan(32);
        self.cached_resonance_curve = axion_solver.compute_conversion_resonance_curve(32);

        self.cached_crossbar_metrics = cross_solver.evaluate_metrics();
        self.cached_s_parameters = cross_solver.compute_s_parameters(32);
        self.cached_linearity = cross_solver.compute_dynamic_range_linearity(32);

        let processor = TopologicalSkinAxionProcessor::new(amp_params, axion_params, cross_params);
        self.cached_audit = processor.audit_system();

        self.last_solve_time_us = start.elapsed().as_micros() as f64;
    }

    /// Primary UI render loop for egui modal window.
    pub fn ui(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut open = self.is_open;
        Window::new("Topological Skin-Effect Microwave Amplifier & Axion Transducer (Phase 446)")
            .open(&mut open)
            .default_width(840.0)
            .default_height(580.0)
            .show(ctx, |ui| {
                self.render_dialog_contents(ui);
            });
        self.is_open = open;
    }

    /// Renders the internal tabs and active panel.
    pub fn render_dialog_contents(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for tab in [
                SkinAxionDialogTab::SkinAmplifierGbz,
                SkinAxionDialogTab::SpatialSkinModes,
                SkinAxionDialogTab::AxionPhononConversion,
                SkinAxionDialogTab::CryogenicReadoutCrossbar,
                SkinAxionDialogTab::AuditTelemetry,
            ] {
                let is_selected = self.active_tab == tab;
                if ui.selectable_label(is_selected, tab.label()).clicked() {
                    self.active_tab = tab;
                }
            }
        });

        ui.separator();

        match self.active_tab {
            SkinAxionDialogTab::SkinAmplifierGbz => self.render_gbz_tab(ui),
            SkinAxionDialogTab::SpatialSkinModes => self.render_spatial_modes_tab(ui),
            SkinAxionDialogTab::AxionPhononConversion => self.render_axion_tab(ui),
            SkinAxionDialogTab::CryogenicReadoutCrossbar => self.render_crossbar_tab(ui),
            SkinAxionDialogTab::AuditTelemetry => self.render_audit_tab(ui),
        }
    }

    fn render_gbz_tab(&mut self, ui: &mut Ui) {
        ui.heading("Non-Hermitian Skin Effect & Generalized Brillouin Zone (GBZ)");
        ui.label("Asymmetric acoustic hopping t_R > t_L driving point-gap winding and directional gain.");
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Forward Hopping t_R:");
            if ui.add(egui::Slider::new(&mut self.forward_hopping_tr_mhz, 4.0..=25.0).suffix(" MHz")).changed() {
                changed = true;
            }
            ui.separator();
            ui.label("Reverse Hopping t_L:");
            if ui.add(egui::Slider::new(&mut self.reverse_hopping_tl_mhz, 0.5..=8.0).suffix(" MHz")).changed() {
                changed = true;
            }
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // GBZ Complex Plane Plot: Re(z) vs Im(z)
        let gbz_pts: PlotPoints = self
            .cached_gbz_spectrum
            .iter()
            .map(|p| [p.z_real, p.z_imag])
            .collect();
        let gbz_line = Line::new("GBZ Boundary Loop |z| = r_GBZ", gbz_pts)
            .color(Color32::from_rgb(56, 189, 248))
            .width(2.2);

        Plot::new("gbz_complex_plane_plot")
            .height(200.0)
            .data_aspect(1.0)
            .x_axis_label("Re(z)")
            .y_axis_label("Im(z)")
            .show(ui, |plot_ui| {
                plot_ui.line(gbz_line);
            });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(format!("Point-Gap Winding: W = {:.1}", self.cached_amp_metrics.point_gap_winding_number));
            ui.separator();
            ui.label(format!("GBZ Radius: {:.3}", self.cached_amp_metrics.gbz_radius));
            ui.separator();
            ui.label(format!("Forward Gain: {:.1} dB", self.cached_amp_metrics.forward_power_gain_db));
            ui.separator();
            ui.label(format!("Reverse Isolation: {:.1} dB", self.cached_amp_metrics.backward_isolation_db));
            ui.separator();
            ui.label(format!("Added Noise: {:.3} quanta", self.cached_amp_metrics.added_noise_quanta));
        });
    }

    fn render_spatial_modes_tab(&mut self, ui: &mut Ui) {
        ui.heading("Spatial Skin Modes & Directional Traveling-Wave Power Profile");
        ui.label("Exponential accumulation of acoustic eigenmodes at the open boundary.");
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Lattice Sites Count N:");
            if ui.add(egui::Slider::new(&mut self.lattice_site_count, 16..=64)).changed() {
                changed = true;
            }
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        let prob_pts: PlotPoints = self
            .cached_spatial_modes
            .iter()
            .map(|p| [p.site_index as f64, p.probability_density])
            .collect();
        let power_pts: PlotPoints = self
            .cached_spatial_modes
            .iter()
            .map(|p| [p.site_index as f64, p.signal_power_db])
            .collect();

        let line_prob = Line::new("Eigenmode Density |Psi(x)|^2", prob_pts)
            .color(Color32::from_rgb(234, 88, 12))
            .width(2.0);
        let line_power = Line::new("Signal Power (dB)", power_pts)
            .color(Color32::from_rgb(34, 197, 94))
            .width(2.0);

        Plot::new("spatial_skin_plot")
            .height(200.0)
            .x_axis_label("Lattice Site Index")
            .y_axis_label("Density / Power")
            .show(ui, |plot_ui| {
                plot_ui.line(line_prob);
                plot_ui.line(line_power);
            });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(format!("Skin Localization Ratio: {:.1}%", self.cached_amp_metrics.skin_localization_ratio * 100.0));
            ui.separator();
            ui.label(format!("Skin Depth: {:.2} sites", self.cached_amp_metrics.skin_depth_sites));
            ui.separator();
            ui.label(format!("Total Lattice Sites: {}", self.lattice_site_count));
        });
    }

    fn render_axion_tab(&mut self, ui: &mut Ui) {
        ui.heading("Coherent Dark-Matter Axion-to-Phonon Transduction");
        ui.label("Primakoff-like conversion of dark matter axions into acoustic modes under high magnetic fields.");
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Magnetic Field B_0:");
            if ui.add(egui::Slider::new(&mut self.magnetic_field_t, 4.0..=16.0).suffix(" T")).changed() {
                changed = true;
            }
            ui.separator();
            ui.label("Axion Mass m_a:");
            if ui.add(egui::Slider::new(&mut self.axion_mass_micro_ev, 4.0..=20.0).suffix(" ueV")).changed() {
                changed = true;
            }
            ui.separator();
            ui.label("Acoustic Q_m:");
            if ui.add(egui::Slider::new(&mut self.acoustic_quality_factor_qm, 1.0e5..=5.0e5).logarithmic(true)).changed() {
                changed = true;
            }
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // Axion Coupling Reach Plot: g_a_gamma_gamma vs Frequency
        let g_exp_pts: PlotPoints = self
            .cached_coupling_scan
            .iter()
            .map(|p| [p.frequency_ghz, p.experimental_coupling_gev_inv * 1e14])
            .collect();
        let ksvz_pts: PlotPoints = self
            .cached_coupling_scan
            .iter()
            .map(|p| [p.frequency_ghz, p.ksvz_target_gev_inv * 1e14])
            .collect();
        let dfsz_pts: PlotPoints = self
            .cached_coupling_scan
            .iter()
            .map(|p| [p.frequency_ghz, p.dfsz_target_gev_inv * 1e14])
            .collect();

        let l_exp = Line::new("Experimental Sensitivity Reach", g_exp_pts)
            .color(Color32::from_rgb(56, 189, 248))
            .width(2.2);
        let l_ksvz = Line::new("KSVZ QCD Axion Model", ksvz_pts)
            .color(Color32::from_rgb(239, 68, 68))
            .width(1.8);
        let l_dfsz = Line::new("DFSZ QCD Axion Model", dfsz_pts)
            .color(Color32::from_rgb(168, 85, 247))
            .width(1.8);

        Plot::new("axion_coupling_plot")
            .height(200.0)
            .x_axis_label("Frequency (GHz)")
            .y_axis_label("Coupling Reach g (x 1e-14 GeV^-1)")
            .show(ui, |plot_ui| {
                plot_ui.line(l_exp);
                plot_ui.line(l_ksvz);
                plot_ui.line(l_dfsz);
            });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(format!("Resonance Frequency: {:.3} GHz", self.cached_axion_metrics.resonance_frequency_ghz));
            ui.separator();
            ui.label(format!("Conversion Efficiency: {:.2e}", self.cached_axion_metrics.conversion_efficiency));
            ui.separator();
            ui.label(format!("Yoctowatt Sensitivity: {:.2e} W/rtHz", self.cached_axion_metrics.yoctowatt_sensitivity_w_sqrt_hz));
            ui.separator();
            ui.label(format!("Coupling Reach: {:.2e} GeV^-1", self.cached_axion_metrics.coupling_sensitivity_gev_inv));
        });
    }

    fn render_crossbar_tab(&mut self, ui: &mut Ui) {
        ui.heading("Cryogenic Microwave Readout Crossbar & Linear Dynamics");
        ui.label("4-Port directional microwave routing, S-parameters, and dynamic range.");
        ui.add_space(6.0);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Operating Temp:");
            if ui.add(egui::Slider::new(&mut self.operating_temp_k, 0.010..=0.100).suffix(" K")).changed() {
                changed = true;
            }
            ui.separator();
            ui.label("Dispersive Shift:");
            if ui.add(egui::Slider::new(&mut self.dispersive_shift_chi_mhz, 1.0..=10.0).suffix(" MHz")).changed() {
                changed = true;
            }
        });

        if changed {
            self.recompute();
        }

        ui.add_space(8.0);

        // S-Parameter Plot
        let s21_pts: PlotPoints = self
            .cached_s_parameters
            .iter()
            .map(|p| [p.frequency_ghz, p.s21_forward_gain_db])
            .collect();
        let s12_pts: PlotPoints = self
            .cached_s_parameters
            .iter()
            .map(|p| [p.frequency_ghz, p.s12_reverse_isolation_db])
            .collect();
        let s11_pts: PlotPoints = self
            .cached_s_parameters
            .iter()
            .map(|p| [p.frequency_ghz, p.s11_return_loss_db])
            .collect();

        let l_s21 = Line::new("S21 Forward Gain", s21_pts).color(Color32::from_rgb(34, 197, 94)).width(2.0);
        let l_s12 = Line::new("S12 Reverse Isolation", s12_pts).color(Color32::from_rgb(239, 68, 68)).width(2.0);
        let l_s11 = Line::new("S11 Return Loss", s11_pts).color(Color32::from_rgb(56, 189, 248)).width(1.5);

        Plot::new("crossbar_s_parameter_plot")
            .height(200.0)
            .x_axis_label("Frequency (GHz)")
            .y_axis_label("S-Parameter (dB)")
            .show(ui, |plot_ui| {
                plot_ui.line(l_s21);
                plot_ui.line(l_s12);
                plot_ui.line(l_s11);
                plot_ui.hline(HLine::new("Isolation Spec", -30.0).color(Color32::GRAY));
            });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(format!("Directivity: {:.1} dB", self.cached_crossbar_metrics.directivity_db));
            ui.separator();
            ui.label(format!("Dynamic Range: {:.1} dB", self.cached_crossbar_metrics.dynamic_range_db));
            ui.separator();
            ui.label(format!("P_1dB Compression: {:.1} dBm", self.cached_crossbar_metrics.p1db_compression_dbm));
            ui.separator();
            ui.label(format!("Dispersive SNR: {:.1} dB", self.cached_crossbar_metrics.dispersive_readout_snr_db));
        });
    }

    fn render_audit_tab(&mut self, ui: &mut Ui) {
        ui.heading("Physics Audit Checklist & Telemetry (10-Point Audit)");
        ui.label("Automated physics verification for Phase 446.");
        ui.add_space(8.0);

        let report = self.cached_audit.clone();

        let items = [
            ("1. Non-Hermitian Point-Gap Winding (|W| = 1.0)", report.point_gap_winding_pass),
            ("2. GBZ Boundary Skin Localization (>= 85.0%)", report.skin_localization_pass),
            ("3. Forward Microwave Power Gain (G_fwd >= 24.0 dB)", report.forward_gain_pass),
            ("4. Directional Backward Isolation (>= 25.0 dB)", report.reverse_isolation_pass),
            ("5. Quantum-Limited Added Noise (n_add <= 0.55 quanta)", report.quantum_added_noise_pass),
            ("6. High Acoustic Resonator Quality Factor (Q_m >= 1.0e5)", report.acoustic_quality_factor_pass),
            ("7. Axion-to-Phonon Conversion Efficiency (>= 1.0e-4)", report.axion_conversion_efficiency_pass),
            ("8. Yoctowatt Sensitivity (P_min <= 1.0e-21 W/rtHz)", report.yoctowatt_sensitivity_pass),
            ("9. Cryogenic Crossbar Directivity (>= 30.0 dB)", report.cryogenic_directivity_pass),
            ("10. Dispersive Readout SNR (SNR >= 18.0 dB)", report.dispersive_readout_snr_pass),
        ];

        for (label, pass) in items {
            ui.horizontal(|ui| {
                let (badge, color) = if pass {
                    ("[PASS]", Color32::from_rgb(34, 197, 94))
                } else {
                    ("[FAIL]", Color32::from_rgb(239, 68, 68))
                };
                ui.label(RichText::new(badge).color(color).monospace());
                ui.label(label);
            });
        }

        ui.add_space(10.0);
        let mut recompute_needed = false;
        ui.horizontal(|ui| {
            let total_str = format!("Total Audit Score: {}/10", report.total_score);
            let score_color = if report.all_passed {
                Color32::from_rgb(34, 197, 94)
            } else {
                Color32::from_rgb(239, 68, 68)
            };
            ui.label(RichText::new(total_str).color(score_color).strong().size(15.0));

            ui.separator();
            ui.label(format!("Last Recompute Latency: {:.1} us", self.last_solve_time_us));

            ui.separator();
            if ui.button("Recompute Physics Solvers").clicked() {
                recompute_needed = true;
            }
        });

        if recompute_needed {
            self.recompute();
        }
    }
}
