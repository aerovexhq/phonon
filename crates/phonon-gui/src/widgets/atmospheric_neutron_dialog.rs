#![deny(unsafe_code)]

//! Interactive Atmospheric Secondary Neutron Spallation Cascade & DO-254 DAL-A SER Dialog.
//!
//! Provides an interactive 5-tab CAD simulation environment:
//! 1. Flight Route & Cosmic Neutron Cascade (renders flight altitude profile, Pfotzer layer, and secondary neutron shower).
//! 2. Secondary Neutron Spectrum & Recoil Kinematics (JESD89A differential flux and 28Si(n,alpha)/(n,p) recoil channels).
//! 3. Avionics SER FIT Rate & DO-254 DAL-A Compliance (Weibull SEU cross-section, TMR/ECC/Lockstep mitigation, and DAL-A pass/fail status).
//! 4. DO-160G Section 22 Lightning Indirect Transients (Waveform 4 / 5A pin injection, TVS/SCR clamping oscilloscope).
//! 5. Clamping Junction Electro-Thermal Dissipation (instantaneous power P(t), dynamic junction temp Delta Tj, and silicon burnout headroom).

use egui::{pos2, vec2, Color32, Context, RichText, Sense, Stroke, StrokeKind, Ui, Window};
use egui_plot::{Legend, Line, Plot, PlotPoints, VLine};
use phonon_solver::atmospheric_neutron::{
    AtmosphericNeutronCoSimulator, AtmosphericNeutronModel, Do254DalLevel, FlightAltitude,
    LightningIndirectSimulator, LightningSeverityLevel, LightningTimeSample, LightningWaveformKind,
    MitigationArchitecture, ProtectionClampDevice, SiliconDeviceParams, SiliconReactionChannel,
    SiliconSpallationEngine, SolarModulation,
};

/// Active tab in the Atmospheric Neutron & DO-254 Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtmosphericNeutronTab {
    FlightRouteCascade,
    NeutronSpectrumKinematics,
    AvionicsSerCompliance,
    LightningTransientScope,
    ThermalDissipationMeter,
}

/// Modal dialog for Atmospheric Neutron Spallation Cascade & Avionics DO-254 DAL-A Co-Simulation.
pub struct AtmosphericNeutronDialog {
    pub is_open: bool,
    pub active_tab: AtmosphericNeutronTab,

    // Flight route parameters
    pub altitude_ft: f64,
    pub latitude_deg: f64,
    pub solar_modulation: SolarModulation,

    // Silicon device & redundancy parameters
    pub memory_mbits: u32,
    pub q_crit_fc: f64,
    pub mitigation_choice: usize, // 0 = Simplex, 1 = ECC SEC-DED, 2 = DMR, 3 = TMR, 4 = Lockstep
    pub scrub_interval_ms: f64,
    pub target_dal: Do254DalLevel,

    // Lightning transient parameters
    pub lightning_waveform: LightningWaveformKind,
    pub lightning_severity: LightningSeverityLevel,
    pub clamp_device_choice: usize, // 0 = TVS Diode, 1 = SCR Snapback
    pub tvs_vbr: f64,
    pub tvs_rdyn: f64,
    pub scr_vt1: f64,
    pub scr_vh: f64,
    pub scr_ron: f64,

    // Simulation engine and cached plot vectors
    pub sim: AtmosphericNeutronCoSimulator,
    pub cached_spectrum: Vec<[f64; 2]>,
    pub cached_alpha_cross_section: Vec<[f64; 2]>,
    pub cached_proton_cross_section: Vec<[f64; 2]>,
    pub cached_lightning_samples: Vec<LightningTimeSample>,
}

impl Default for AtmosphericNeutronDialog {
    fn default() -> Self {
        let sim = AtmosphericNeutronCoSimulator::new_fast();

        // Pre-seeded lightweight vectors for instant cold boot
        let cached_spectrum = vec![
            [1.0, 1.2e-3],
            [10.0, 4.5e-4],
            [100.0, 3.8e-4],
            [1000.0, 2.1e-5],
            [10000.0, 1.8e-7],
        ];

        let cached_alpha_cross_section = vec![
            [2.5, 0.0],
            [10.0, 120.0],
            [14.0, 180.0],
            [20.0, 75.0],
            [30.0, 10.0],
        ];

        let cached_proton_cross_section = vec![
            [3.5, 0.0],
            [12.0, 150.0],
            [17.0, 240.0],
            [25.0, 90.0],
            [35.0, 15.0],
        ];

        let cached_lightning_samples = sim.lightning_simulator.simulate_transient(300.0, 80);

        Self {
            is_open: false,
            active_tab: AtmosphericNeutronTab::FlightRouteCascade,
            altitude_ft: 39_000.0,
            latitude_deg: 45.0,
            solar_modulation: SolarModulation::SolarModerate,
            memory_mbits: 64,
            q_crit_fc: 0.45,
            mitigation_choice: 3, // TMR
            scrub_interval_ms: 50.0,
            target_dal: Do254DalLevel::DalA,
            lightning_waveform: LightningWaveformKind::Waveform4,
            lightning_severity: LightningSeverityLevel::Level4,
            clamp_device_choice: 0, // TVS Diode
            tvs_vbr: 12.0,
            tvs_rdyn: 0.035,
            scr_vt1: 14.0,
            scr_vh: 3.5,
            scr_ron: 0.020,
            sim,
            cached_spectrum,
            cached_alpha_cross_section,
            cached_proton_cross_section,
            cached_lightning_samples,
        }
    }
}

// Private helper marker
impl AtmosphericNeutronDialog {
    #[allow(dead_code)]
    fn dummy(&self) {}
}

impl AtmosphericNeutronDialog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fast non-blocking constructor guaranteeing sub-millisecond initialization for cold boot.
    pub fn new_fast() -> Self {
        Self::default()
    }

    /// Recompute all solver physics, spectra, and lightning waveforms.
    pub fn recompute_sim(&mut self) {
        // Build neutron model
        let alt = FlightAltitude::Custom(self.altitude_ft.round() as u32);
        self.sim.neutron_model = AtmosphericNeutronModel::new(alt, self.latitude_deg, self.solar_modulation);

        // Build device and mitigation
        let device = SiliconDeviceParams {
            q_crit_fc: self.q_crit_fc,
            total_bits: (self.memory_mbits as u64) * 1024 * 1024,
            sigma_sat_cm2_per_bit: 1.25e-14,
            weibull_e0_mev: 4.5,
            weibull_w_mev: 28.0,
            weibull_s: 1.45,
        };

        let mitigation = match self.mitigation_choice {
            0 => MitigationArchitecture::Simplex,
            1 => MitigationArchitecture::EccSecDed,
            2 => MitigationArchitecture::DualModularRedundancy,
            3 => MitigationArchitecture::TripleModularRedundancy,
            _ => MitigationArchitecture::LockstepWithScrubbing {
                scrub_interval_s: self.scrub_interval_ms * 0.001,
            },
        };

        self.sim.spallation_engine = SiliconSpallationEngine::new(
            self.sim.neutron_model.clone(),
            device,
            mitigation,
            self.target_dal,
        );

        // Build lightning clamp
        let protection = match self.clamp_device_choice {
            0 => ProtectionClampDevice::TvsDiode {
                v_br: self.tvs_vbr,
                r_dyn: self.tvs_rdyn,
            },
            _ => ProtectionClampDevice::ScrSnapback {
                v_t1: self.scr_vt1,
                v_h: self.scr_vh,
                r_on: self.scr_ron,
            },
        };

        self.sim.lightning_simulator = LightningIndirectSimulator::new(
            self.lightning_waveform,
            self.lightning_severity,
            protection,
        );

        // Recompute simulation report
        self.sim.recompute();

        // Sample neutron energy spectrum
        self.cached_spectrum = self
            .sim
            .neutron_model
            .sample_spectrum(60)
            .into_iter()
            .map(|(e, f)| [e, f])
            .collect();

        // Sample reaction cross sections
        let alpha_chan = SiliconReactionChannel::AlphaRecoil;
        let proton_chan = SiliconReactionChannel::ProtonRecoil;

        self.cached_alpha_cross_section = (0..60)
            .map(|i| {
                let e = 1.0 + (i as f64) * 0.75;
                [e, alpha_chan.cross_section_mb(e)]
            })
            .collect();

        self.cached_proton_cross_section = (0..60)
            .map(|i| {
                let e = 1.0 + (i as f64) * 0.75;
                [e, proton_chan.cross_section_mb(e)]
            })
            .collect();

        // Sample lightning transient
        self.cached_lightning_samples = self.sim.lightning_simulator.simulate_transient(300.0, 100);
    }

    /// Render modal UI window (standard alias).
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Render modal UI window.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new("Atmospheric Secondary Neutron Cascade & DO-254 DAL-A Co-Simulator")
            .open(&mut is_open)
            .default_size(vec2(980.0, 680.0))
            .min_size(vec2(860.0, 560.0))
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    pub fn render_content(&mut self, ui: &mut Ui) {
        // Tab bar
        ui.horizontal(|ui| {
            if ui
                .selectable_label(
                    self.active_tab == AtmosphericNeutronTab::FlightRouteCascade,
                    "Flight Route & Cascade",
                )
                .clicked()
            {
                self.active_tab = AtmosphericNeutronTab::FlightRouteCascade;
            }
            if ui
                .selectable_label(
                    self.active_tab == AtmosphericNeutronTab::NeutronSpectrumKinematics,
                    "Neutron Spectrum & Recoil",
                )
                .clicked()
            {
                self.active_tab = AtmosphericNeutronTab::NeutronSpectrumKinematics;
            }
            if ui
                .selectable_label(
                    self.active_tab == AtmosphericNeutronTab::AvionicsSerCompliance,
                    "DO-254 DAL-A Certification",
                )
                .clicked()
            {
                self.active_tab = AtmosphericNeutronTab::AvionicsSerCompliance;
            }
            if ui
                .selectable_label(
                    self.active_tab == AtmosphericNeutronTab::LightningTransientScope,
                    "DO-160G Lightning Scope",
                )
                .clicked()
            {
                self.active_tab = AtmosphericNeutronTab::LightningTransientScope;
            }
            if ui
                .selectable_label(
                    self.active_tab == AtmosphericNeutronTab::ThermalDissipationMeter,
                    "Electro-Thermal Dissipation",
                )
                .clicked()
            {
                self.active_tab = AtmosphericNeutronTab::ThermalDissipationMeter;
            }
        });

        ui.separator();

        // Active tab rendering
        match self.active_tab {
            AtmosphericNeutronTab::FlightRouteCascade => self.render_flight_route_tab(ui),
            AtmosphericNeutronTab::NeutronSpectrumKinematics => self.render_neutron_spectrum_tab(ui),
            AtmosphericNeutronTab::AvionicsSerCompliance => self.render_avionics_ser_tab(ui),
            AtmosphericNeutronTab::LightningTransientScope => self.render_lightning_scope_tab(ui),
            AtmosphericNeutronTab::ThermalDissipationMeter => self.render_thermal_meter_tab(ui),
        }

        ui.separator();
        self.render_telemetry_strip(ui);
    }

    fn render_flight_route_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("Atmospheric Flight Altitude & Cosmic Secondary Cascade");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Recompute Environment").clicked() {
                    self.recompute_sim();
                }
            });
        });

        ui.add_space(4.0);

        // Preset buttons
        ui.horizontal(|ui| {
            ui.label(RichText::new("Flight Presets:").strong());
            if ui.button("Transatlantic (FL390, 52 deg N)").clicked() {
                self.altitude_ft = 39_000.0;
                self.latitude_deg = 52.0;
                self.solar_modulation = SolarModulation::SolarModerate;
                self.recompute_sim();
            }
            if ui.button("Polar Route (FL410, 75 deg N)").clicked() {
                self.altitude_ft = 41_000.0;
                self.latitude_deg = 75.0;
                self.solar_modulation = SolarModulation::SolarMinimum;
                self.recompute_sim();
            }
            if ui.button("Equatorial (FL350, 0 deg)").clicked() {
                self.altitude_ft = 35_000.0;
                self.latitude_deg = 0.0;
                self.solar_modulation = SolarModulation::SolarMaximum;
                self.recompute_sim();
            }
            if ui.button("High-Altitude Recon (FL500, 60 deg N)").clicked() {
                self.altitude_ft = 50_000.0;
                self.latitude_deg = 60.0;
                self.solar_modulation = SolarModulation::SolarMinimum;
                self.recompute_sim();
            }
        });

        ui.add_space(6.0);

        // Control Sliders
        ui.horizontal(|ui| {
            let mut changed = false;
            ui.label("Altitude (ft):");
            changed |= ui
                .add(egui::Slider::new(&mut self.altitude_ft, 0.0..=60_000.0).step_by(500.0))
                .changed();

            ui.separator();
            ui.label("Latitude (deg):");
            changed |= ui
                .add(egui::Slider::new(&mut self.latitude_deg, -90.0..=90.0).step_by(1.0))
                .changed();

            ui.separator();
            ui.label("Solar Cycle:");
            let mut solar_idx = match self.solar_modulation {
                SolarModulation::SolarMinimum => 0,
                SolarModulation::SolarModerate => 1,
                SolarModulation::SolarMaximum => 2,
            };
            egui::ComboBox::from_id_salt("solar_cycle_combo")
                .selected_text(match solar_idx {
                    0 => "Solar Minimum (Max GCR)",
                    1 => "Solar Moderate",
                    _ => "Solar Maximum (Min GCR)",
                })
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut solar_idx, 0, "Solar Minimum (Max GCR)").clicked() {
                        self.solar_modulation = SolarModulation::SolarMinimum;
                        changed = true;
                    }
                    if ui.selectable_value(&mut solar_idx, 1, "Solar Moderate").clicked() {
                        self.solar_modulation = SolarModulation::SolarModerate;
                        changed = true;
                    }
                    if ui.selectable_value(&mut solar_idx, 2, "Solar Maximum (Min GCR)").clicked() {
                        self.solar_modulation = SolarModulation::SolarMaximum;
                        changed = true;
                    }
                });

            if changed {
                self.recompute_sim();
            }
        });

        ui.add_space(8.0);

        // 2D Canvas rendering altitude profile & cosmic neutron shower
        let (response, painter) =
            ui.allocate_painter(vec2(ui.available_width(), 320.0), Sense::hover());
        let rect = response.rect;

        // Background sky gradient
        painter.rect_filled(rect, 4.0, Color32::from_rgb(12, 16, 28));
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(45, 60, 90)),
            StrokeKind::Outside,
        );

        let cx = rect.center().x;
        let ground_y = rect.bottom() - 30.0;
        let top_y = rect.top() + 30.0;
        let height_span = ground_y - top_y;

        // Draw ground level
        painter.line_segment(
            [pos2(rect.left() + 20.0, ground_y), pos2(rect.right() - 20.0, ground_y)],
            Stroke::new(2.0, Color32::from_rgb(70, 95, 60)),
        );
        painter.text(
            pos2(rect.left() + 30.0, ground_y + 12.0),
            egui::Align2::LEFT_CENTER,
            "Sea Level (0 ft, 1033 g/cm^2)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(150, 180, 150),
        );

        // Draw Pfotzer maximum layer (~60,000 ft / 18 km)
        let pfotzer_y = ground_y - (60_000.0 / 60_000.0) * height_span;
        painter.line_segment(
            [pos2(rect.left() + 20.0, pfotzer_y), pos2(rect.right() - 20.0, pfotzer_y)],
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(200, 150, 50, 100)),
        );
        painter.text(
            pos2(rect.right() - 30.0, pfotzer_y - 10.0),
            egui::Align2::RIGHT_CENTER,
            "Pfotzer Maximum (~60,000 ft, 100 g/cm^2)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(220, 180, 100),
        );

        // Aircraft flight altitude position
        let alt_fraction = (self.altitude_ft / 60_000.0).clamp(0.0, 1.0);
        let plane_y = ground_y - (alt_fraction as f32) * height_span;

        // Draw altitude dashed line
        painter.line_segment(
            [pos2(rect.left() + 20.0, plane_y), pos2(rect.right() - 20.0, plane_y)],
            Stroke::new(1.5, Color32::from_rgb(80, 140, 220)),
        );

        // Draw Aircraft symbol
        let plane_pos = pos2(cx, plane_y);
        painter.circle_filled(plane_pos, 7.0, Color32::from_rgb(100, 180, 255));
        painter.line_segment(
            [pos2(plane_pos.x - 22.0, plane_pos.y), pos2(plane_pos.x + 22.0, plane_pos.y)],
            Stroke::new(3.0, Color32::from_rgb(220, 240, 255)),
        );
        painter.line_segment(
            [pos2(plane_pos.x, plane_pos.y - 14.0), pos2(plane_pos.x, plane_pos.y + 14.0)],
            Stroke::new(2.5, Color32::from_rgb(220, 240, 255)),
        );

        painter.text(
            pos2(plane_pos.x + 30.0, plane_pos.y - 8.0),
            egui::Align2::LEFT_CENTER,
            format!("Aircraft Cruise: FL{:.0} ({:.0} ft)", self.altitude_ft / 100.0, self.altitude_ft),
            egui::FontId::proportional(12.0),
            Color32::from_rgb(140, 210, 255),
        );

        // Draw Cosmic Ray Primary Proton & Secondary Neutron Shower
        let proton_top = pos2(cx - 120.0, top_y);
        let spallation_site = pos2(cx - 40.0, pfotzer_y + 15.0);
        painter.line_segment([proton_top, spallation_site], Stroke::new(2.5, Color32::from_rgb(255, 90, 80)));
        painter.text(
            pos2(proton_top.x, proton_top.y - 12.0),
            egui::Align2::CENTER_CENTER,
            "Primary Cosmic Ray (Proton / Iron, > 10 GeV)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(255, 130, 120),
        );

        // Secondary Neutron shower cascades hitting the aircraft
        let n_targets = [
            pos2(plane_pos.x - 60.0, plane_pos.y),
            pos2(plane_pos.x - 10.0, plane_pos.y),
            pos2(plane_pos.x + 20.0, plane_pos.y),
            pos2(plane_pos.x + 80.0, plane_pos.y),
        ];

        for (idx, target) in n_targets.iter().enumerate() {
            let color = match idx % 3 {
                0 => Color32::from_rgba_unmultiplied(120, 220, 255, 180),
                1 => Color32::from_rgba_unmultiplied(255, 200, 80, 180),
                _ => Color32::from_rgba_unmultiplied(180, 140, 255, 180),
            };
            painter.line_segment([spallation_site, *target], Stroke::new(1.5, color));
            painter.circle_filled(*target, 3.0, Color32::from_rgb(255, 255, 255));
        }

        painter.text(
            pos2(cx - 30.0, spallation_site.y + 14.0),
            egui::Align2::LEFT_CENTER,
            "Nuclear Spallation Vertex in N2/O2 Atmosphere",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(255, 220, 140),
        );
    }

    fn render_neutron_spectrum_tab(&mut self, ui: &mut Ui) {
        ui.heading("Secondary Neutron Differential Spectrum (JESD89A) & Recoil Cross Sections");
        ui.label(
            "Differential flux dPhi/dE showing evaporation peak (1-3 MeV), 100 MeV spallation resonance, and nuclear reaction cross-sections.",
        );

        ui.add_space(6.0);

        ui.columns(2, |cols| {
            // Column 1: JEDEC JESD89A Differential Spectrum
            cols[0].vertical(|ui| {
                ui.label(RichText::new("Differential Neutron Flux dPhi/dE vs Energy (MeV)").strong());

                let points = PlotPoints::new(self.cached_spectrum.iter().map(|p| [p[0].log10(), p[1].log10()]).collect());
                let line = Line::new("JESD89A Flight Spectrum", points)
                    .color(Color32::from_rgb(80, 180, 255))
                    .width(2.0);

                Plot::new("neutron_spectrum_plot")
                    .height(280.0)
                    .legend(Legend::default())
                    .x_axis_label("Log10 Energy (MeV)")
                    .y_axis_label("Log10 Flux n/(cm^2*s*MeV)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(line);
                        // Evaporation peak marker (log10(2) ~ 0.3)
                        plot_ui.vline(VLine::new("Evap Peak (2 MeV)", 0.3).color(Color32::from_rgb(255, 180, 80)));
                        // Spallation peak marker (log10(100) = 2.0)
                        plot_ui.vline(VLine::new("Spallation Peak (100 MeV)", 2.0).color(Color32::from_rgb(255, 100, 100)));
                    });
            });

            // Column 2: Silicon Reaction Cross-Sections (mb)
            cols[1].vertical(|ui| {
                ui.label(RichText::new("Silicon Nuclear Recoil Cross Section (mb) vs Energy").strong());

                let alpha_pts = PlotPoints::new(self.cached_alpha_cross_section.iter().map(|p| [p[0], p[1]]).collect());
                let alpha_line = Line::new("28Si(n, a)25Mg (Peak 180 mb)", alpha_pts)
                    .color(Color32::from_rgb(255, 140, 60))
                    .width(2.0);

                let proton_pts = PlotPoints::new(self.cached_proton_cross_section.iter().map(|p| [p[0], p[1]]).collect());
                let proton_line = Line::new("28Si(n, p)28Al (Peak 240 mb)", proton_pts)
                    .color(Color32::from_rgb(180, 100, 255))
                    .width(2.0);

                Plot::new("silicon_cross_section_plot")
                    .height(280.0)
                    .legend(Legend::default())
                    .x_axis_label("Incident Neutron Energy (MeV)")
                    .y_axis_label("Reaction Cross Section (mb)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(alpha_line);
                        plot_ui.line(proton_line);
                    });
            });
        });
    }

    fn render_avionics_ser_tab(&mut self, ui: &mut Ui) {
        ui.heading("Avionics Soft Error Rate (SER) & DO-254 DAL-A Airworthiness Certification");
        ui.label(
            "Evaluation of single event upset (SEU) cross-sections, unmitigated FIT rates, and fault mitigation architectures for FAA/EASA airworthiness compliance.",
        );

        ui.add_space(6.0);

        let report = self.sim.latest_report().clone();

        // Compliance Banner
        let (banner_text, banner_bg, banner_fg) = if report.is_dal_a_compliant {
            (
                format!(
                    "COMPLIANT: DO-254 DAL-A Certified (Hazard Rate: {:.2e} / flight hr, Margin: +{:.2} dB)",
                    report.mitigated_failure_rate_per_hour, report.dal_margin_db
                ),
                Color32::from_rgb(20, 60, 30),
                Color32::from_rgb(120, 255, 140),
            )
        } else {
            (
                format!(
                    "NON-COMPLIANT: Fails DO-254 DAL-A (Hazard Rate: {:.2e} / flight hr, Target: < 1.00e-9 / hr)",
                    report.mitigated_failure_rate_per_hour
                ),
                Color32::from_rgb(70, 20, 20),
                Color32::from_rgb(255, 140, 140),
            )
        };

        ui.add(
            egui::Label::new(
                RichText::new(banner_text)
                    .strong()
                    .color(banner_fg)
                    .background_color(banner_bg),
            )
            .selectable(false),
        );

        ui.add_space(10.0);

        ui.columns(2, |cols| {
            // Configuration controls
            cols[0].vertical(|ui| {
                ui.label(RichText::new("Target Avionics Device & Redundancy Architecture").strong());

                let mut changed = false;

                ui.horizontal(|ui| {
                    ui.label("Memory Capacity (Mbits):");
                    changed |= ui.add(egui::Slider::new(&mut self.memory_mbits, 4..=1024).step_by(4.0)).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Critical Charge Qcrit (fC):");
                    changed |= ui.add(egui::Slider::new(&mut self.q_crit_fc, 0.15..=2.0).step_by(0.05)).changed();
                });

                ui.separator();
                ui.label(RichText::new("Mitigation Architecture:").strong());

                changed |= ui.radio_value(&mut self.mitigation_choice, 0, "Simplex (Unmitigated)").changed();
                changed |= ui.radio_value(&mut self.mitigation_choice, 1, "ECC SEC-DED (Word parity)").changed();
                changed |= ui.radio_value(&mut self.mitigation_choice, 2, "Dual Modular Redundancy (DMR, Fail-Silent)").changed();
                changed |= ui.radio_value(&mut self.mitigation_choice, 3, "Triple Modular Redundancy (TMR, Majority Voter)").changed();
                changed |= ui.radio_value(&mut self.mitigation_choice, 4, "Dual-Core Lockstep + Memory Scrubbing").changed();

                if self.mitigation_choice == 4 {
                    ui.horizontal(|ui| {
                        ui.label("Scrub Interval (ms):");
                        changed |= ui.add(egui::Slider::new(&mut self.scrub_interval_ms, 5.0..=500.0)).changed();
                    });
                }

                ui.separator();
                ui.label(RichText::new("Target DO-254 Assurance Level:").strong());
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.target_dal == Do254DalLevel::DalA, "DAL-A (< 1e-9/hr)").clicked() {
                        self.target_dal = Do254DalLevel::DalA;
                        changed = true;
                    }
                    if ui.selectable_label(self.target_dal == Do254DalLevel::DalB, "DAL-B (< 1e-7/hr)").clicked() {
                        self.target_dal = Do254DalLevel::DalB;
                        changed = true;
                    }
                    if ui.selectable_label(self.target_dal == Do254DalLevel::DalC, "DAL-C (< 1e-5/hr)").clicked() {
                        self.target_dal = Do254DalLevel::DalC;
                        changed = true;
                    }
                });

                if changed {
                    self.recompute_sim();
                }
            });

            // Diagnostic Telemetry Readouts
            cols[1].vertical(|ui| {
                ui.label(RichText::new("Reliability & Hazard Assessment Telemetry").strong());

                ui.group(|ui| {
                    ui.label(format!("Raw Unmitigated Chip SER: {:.1} FIT", report.raw_chip_ser_fit));
                    ui.label(format!(
                        "Raw Simplex Failure Rate: {:.2e} / flight hour",
                        report.raw_chip_ser_fit * 1.0e-9
                    ));
                    ui.separator();
                    ui.label(format!(
                        "Mitigated Catastrophic Rate: {:.2e} / flight hour",
                        report.mitigated_failure_rate_per_hour
                    ));
                    ui.label(format!(
                        "Equivalent Mitigated SER: {:.3e} FIT",
                        report.mitigated_failure_rate_per_hour * 1.0e9
                    ));
                    ui.label(format!("DO-254 Safety Margin: {:.2} dB", report.dal_margin_db));
                    ui.separator();
                    ui.label(format!(
                        "Mean Time Between Failures (MTBF): {:.2e} flight hours",
                        1.0 / report.mitigated_failure_rate_per_hour.max(1.0e-25)
                    ));
                });
            });
        });
    }

    fn render_lightning_scope_tab(&mut self, ui: &mut Ui) {
        ui.heading("DO-160G Section 22 Lightning Indirect Transient Oscilloscope");
        ui.label(
            "Pin injection transient testing: Waveform 4 (Voltage Impulse) & Waveform 5A (Current Surge) with TVS/SCR on-chip clamping.",
        );

        ui.add_space(6.0);

        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Waveform:");
            if ui
                .selectable_label(
                    self.lightning_waveform == LightningWaveformKind::Waveform4,
                    "Waveform 4 (6.4 / 69 us)",
                )
                .clicked()
            {
                self.lightning_waveform = LightningWaveformKind::Waveform4;
                changed = true;
            }
            if ui
                .selectable_label(
                    self.lightning_waveform == LightningWaveformKind::Waveform5A,
                    "Waveform 5A (40 / 120 us)",
                )
                .clicked()
            {
                self.lightning_waveform = LightningWaveformKind::Waveform5A;
                changed = true;
            }

            ui.separator();
            ui.label("Severity:");
            let mut sev_idx = match self.lightning_severity {
                LightningSeverityLevel::Level1 => 1,
                LightningSeverityLevel::Level2 => 2,
                LightningSeverityLevel::Level3 => 3,
                LightningSeverityLevel::Level4 => 4,
                LightningSeverityLevel::Level5 => 5,
            };
            egui::ComboBox::from_id_salt("lightning_severity_combo")
                .selected_text(format!("Level {} ({:.0}V / {:.0}A)", sev_idx, self.lightning_severity.peak_volt_amp().0, self.lightning_severity.peak_volt_amp().1))
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut sev_idx, 1, "Level 1 (100V / 20A)").clicked() {
                        self.lightning_severity = LightningSeverityLevel::Level1;
                        changed = true;
                    }
                    if ui.selectable_value(&mut sev_idx, 2, "Level 2 (250V / 50A)").clicked() {
                        self.lightning_severity = LightningSeverityLevel::Level2;
                        changed = true;
                    }
                    if ui.selectable_value(&mut sev_idx, 3, "Level 3 (600V / 120A)").clicked() {
                        self.lightning_severity = LightningSeverityLevel::Level3;
                        changed = true;
                    }
                    if ui.selectable_value(&mut sev_idx, 4, "Level 4 (1000V / 500A)").clicked() {
                        self.lightning_severity = LightningSeverityLevel::Level4;
                        changed = true;
                    }
                    if ui.selectable_value(&mut sev_idx, 5, "Level 5 (1600V / 1600A)").clicked() {
                        self.lightning_severity = LightningSeverityLevel::Level5;
                        changed = true;
                    }
                });

            ui.separator();
            ui.label("Clamp Device:");
            if ui.selectable_label(self.clamp_device_choice == 0, "TVS Diode").clicked() {
                self.clamp_device_choice = 0;
                changed = true;
            }
            if ui.selectable_label(self.clamp_device_choice == 1, "SCR Snapback").clicked() {
                self.clamp_device_choice = 1;
                changed = true;
            }
        });

        if changed {
            self.recompute_sim();
        }

        ui.add_space(8.0);

        // Oscilloscope Plot
        let voc_pts = PlotPoints::new(
            self.cached_lightning_samples
                .iter()
                .map(|s| [s.time_us, s.source_voltage_v])
                .collect(),
        );
        let voc_line = Line::new("Voc Generator Open Circuit (V)", voc_pts)
            .color(Color32::from_rgb(255, 90, 80))
            .width(1.5);

        let vclamp_pts = PlotPoints::new(
            self.cached_lightning_samples
                .iter()
                .map(|s| [s.time_us, s.clamped_voltage_v])
                .collect(),
        );
        let vclamp_line = Line::new("Vclamp Terminal Voltage (V)", vclamp_pts)
            .color(Color32::from_rgb(80, 255, 120))
            .width(2.5);

        let isurge_pts = PlotPoints::new(
            self.cached_lightning_samples
                .iter()
                .map(|s| [s.time_us, s.surge_current_a])
                .collect(),
        );
        let isurge_line = Line::new("Isurge Clamp Current (A)", isurge_pts)
            .color(Color32::from_rgb(255, 200, 60))
            .width(1.8);

        Plot::new("lightning_scope_plot")
            .height(290.0)
            .legend(Legend::default())
            .x_axis_label("Time (microseconds)")
            .y_axis_label("Voltage (V) / Current (A)")
            .show(ui, |plot_ui| {
                plot_ui.line(voc_line);
                plot_ui.line(vclamp_line);
                plot_ui.line(isurge_line);
            });
    }

    fn render_thermal_meter_tab(&mut self, ui: &mut Ui) {
        ui.heading("Clamping Junction Electro-Thermal Dissipation & Silicon Burnout Headroom");
        ui.label(
            "Transient junction thermal heating Delta Tj(t) and comparison against silicon physical damage threshold (350 deg C).",
        );

        ui.add_space(6.0);

        let report = self.sim.latest_report().clone();

        ui.columns(2, |cols| {
            // Plot 1: Instantaneous Power P(t) (kW)
            cols[0].vertical(|ui| {
                ui.label(RichText::new("Instantaneous Clamping Power P(t) [kW]").strong());

                let power_pts = PlotPoints::new(
                    self.cached_lightning_samples
                        .iter()
                        .map(|s| [s.time_us, s.instantaneous_power_w * 1.0e-3])
                        .collect(),
                );
                let power_line = Line::new("Power Dissipation (kW)", power_pts)
                    .color(Color32::from_rgb(255, 120, 60))
                    .width(2.0);

                Plot::new("thermal_power_plot")
                    .height(260.0)
                    .legend(Legend::default())
                    .x_axis_label("Time (us)")
                    .y_axis_label("Power (kW)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(power_line);
                    });
            });

            // Plot 2: Dynamic Junction Temperature Tj(t) (deg C)
            cols[1].vertical(|ui| {
                ui.label(RichText::new("Junction Temperature Tj(t) [deg C]").strong());

                let temp_pts = PlotPoints::new(
                    self.cached_lightning_samples
                        .iter()
                        .map(|s| [s.time_us, 70.0 + s.temp_rise_deg_c])
                        .collect(),
                );
                let temp_line = Line::new("Tj Junction Temp (deg C)", temp_pts)
                    .color(Color32::from_rgb(255, 60, 80))
                    .width(2.5);

                Plot::new("junction_temp_plot")
                    .height(260.0)
                    .legend(Legend::default())
                    .x_axis_label("Time (us)")
                    .y_axis_label("Temperature (deg C)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(temp_line);
                        // Failure threshold line at 350 deg C
                        plot_ui.vline(VLine::new("Failure Limit (350 C)", 350.0).color(Color32::from_rgb(255, 80, 80)));
                    });
            });
        });

        ui.add_space(8.0);

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Thermal Burnout Assessment:").strong());
                ui.label(format!("Peak Junction Temperature: {:.1} deg C", report.lightning_peak_junction_temp_c));
                ui.separator();
                ui.label(format!("Silicon Failure Threshold: 350.0 deg C"));
                ui.separator();
                let margin_color = if report.lightning_thermal_margin_c > 50.0 {
                    Color32::from_rgb(80, 240, 100)
                } else if report.lightning_thermal_margin_c > 0.0 {
                    Color32::from_rgb(240, 200, 60)
                } else {
                    Color32::from_rgb(255, 60, 60)
                };
                ui.label(
                    RichText::new(format!("Headroom Margin: +{:.1} deg C", report.lightning_thermal_margin_c))
                        .strong()
                        .color(margin_color),
                );
            });
        });
    }

    fn render_telemetry_strip(&self, ui: &mut Ui) {
        let rep = self.sim.latest_report();
        ui.horizontal(|ui| {
            ui.label(RichText::new("Telemetry:").strong().color(Color32::from_rgb(180, 200, 230)));
            ui.label(format!("Alt: {}", rep.altitude_label));
            ui.separator();
            ui.label(format!("Flux Accel: {:.1}x", rep.altitude_acceleration));
            ui.separator();
            ui.label(format!("Flux >10MeV: {:.0} n/(cm^2*h)", rep.total_flux_gt_10mev_per_hour));
            ui.separator();
            ui.label(format!("Raw SER: {:.0} FIT", rep.raw_chip_ser_fit));
            ui.separator();
            let dal_color = if rep.is_dal_a_compliant {
                Color32::from_rgb(80, 240, 120)
            } else {
                Color32::from_rgb(255, 80, 80)
            };
            ui.label(
                RichText::new(if rep.is_dal_a_compliant {
                    "DO-254 DAL-A: PASS"
                } else {
                    "DO-254 DAL-A: FAIL"
                })
                .strong()
                .color(dal_color),
            );
            ui.separator();
            ui.label(format!("Peak Vclamp: {:.1} V", rep.lightning_peak_v_clamp));
            ui.separator();
            ui.label(format!("Peak Tj: {:.1} deg C", rep.lightning_peak_junction_temp_c));
        });
    }
}
