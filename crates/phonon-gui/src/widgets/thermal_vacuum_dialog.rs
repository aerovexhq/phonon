#![deny(unsafe_code)]

//! Interactive Aerospace Thermal-Vacuum Radiation Dissipation & Orbital Cycling Co-Simulator Dialog.
//!
//! Provides an interactive 5-tab CAD simulation environment:
//! 1. Orbital Environment & Radiator Geometry (renders 2D orbital trajectory, Earth/Moon, solar flux, and eclipse shadow).
//! 2. Stefan-Boltzmann Vacuum Dissipation (zero-convection h=0, T^4 cooling, Gold/Paint/Anodize/MLI surface emissivity).
//! 3. Orbital Rapid Thermal Cycling (solar-eclipse transient temperature swing Delta T over 1 orbital period).
//! 4. Micro-Bump Viscoplastic Creep & Fatigue (Coffin-Manson Delta gamma_p, SAC305/SnPb/Indium, mission lifetime years).
//! 5. Cryogenic Carrier Freeze-Out & Kink TCAD (dopant freeze-out n(T), subthreshold slope < 20 mV/dec, substrate kink).

use egui::{pos2, vec2, Color32, Context, RichText, Sense, Stroke, StrokeKind, Ui, Window};
use egui_plot::{Legend, Line, Plot, PlotPoints, VLine};
use phonon_solver::thermal_vacuum::{
    CryogenicDopantKind, OrbitalMissionKind, SolderAlloyKind, SurfaceCoatingKind,
    ThermalVacuumCoSimulator,
};

/// Active tab in the Thermal-Vacuum Co-Simulator Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalVacuumTab {
    OrbitalEnvironment,
    StefanBoltzmannCooldown,
    OrbitalCyclingProfile,
    MicroBumpFatigue,
    CryogenicFreezeoutKink,
}

/// Modal dialog for Aerospace Thermal-Vacuum Radiation & Orbital Cycling Co-Simulation.
pub struct ThermalVacuumDialog {
    pub is_open: bool,
    pub active_tab: ThermalVacuumTab,

    // Environment & mission parameters
    pub mission_choice: usize, // 0 = LEO, 1 = GEO, 2 = Lunar, 3 = Deep Space
    pub coating_choice: usize, // 0 = White Paint, 1 = Polished Gold, 2 = Black Anodize, 3 = MLI, 4 = Bare Silicon
    pub radiator_area_m2: f64,
    pub package_heat_capacity: f64,
    pub internal_heat_w: f64,

    // Micro-bump thermo-mechanics
    pub solder_alloy_choice: usize, // 0 = SAC305, 1 = Sn63Pb37, 2 = Pure Indium
    pub dnp_mm: f64,
    pub standoff_um: f64,
    pub substrate_cte: f64,
    pub underfill_coupling: f64,

    // Cryogenic semiconductor TCAD
    pub dopant_choice: usize, // 0 = Phosphorus, 1 = Boron, 2 = Arsenic, 3 = Antimony, 4 = Indium
    pub nominal_doping_log: f64, // log10(N_D) e.g. 17.0 for 1e17 cm^-3
    pub cryo_temp_k: f64,
    pub cryo_vgs: f64,

    // Simulation engine and cached plot vectors
    pub sim: ThermalVacuumCoSimulator,
    pub cached_cooldown: Vec<[f64; 2]>,
    pub cached_orbital_profile: Vec<[f64; 2]>,
    pub cached_freezeout: Vec<[f64; 2]>,
    pub cached_subthreshold: Vec<[f64; 2]>,
    pub cached_id_vds: Vec<[f64; 2]>,
}

impl Default for ThermalVacuumDialog {
    fn default() -> Self {
        let sim = ThermalVacuumCoSimulator::new_fast();

        // Pre-seeded lightweight vectors for instant cold boot (<1ms)
        let cached_cooldown = vec![
            [0.0, 76.8],
            [900.0, 32.4],
            [1800.0, -12.5],
            [2700.0, -42.8],
            [3600.0, -58.2],
        ];

        let cached_orbital_profile = vec![
            [0.0, 42.6],
            [25.0, 41.2],
            [55.0, 39.8],
            [65.0, -25.4],
            [85.0, -68.4],
            [92.5, 42.6],
        ];

        let cached_freezeout = vec![
            [4.2, 0.0001],
            [20.0, 0.008],
            [50.0, 0.085],
            [77.0, 0.342],
            [150.0, 0.820],
            [300.0, 0.985],
        ];

        let cached_subthreshold = vec![
            [4.2, 3.6],
            [20.0, 5.2],
            [50.0, 11.8],
            [77.0, 17.8],
            [150.0, 36.2],
            [300.0, 68.5],
        ];

        let cached_id_vds = vec![
            [0.0, 0.0],
            [0.4, 0.62],
            [0.8, 0.98],
            [1.0, 1.05],
            [1.4, 1.48],
            [2.0, 2.35],
        ];

        Self {
            is_open: false,
            active_tab: ThermalVacuumTab::OrbitalEnvironment,
            mission_choice: 0, // LEO
            coating_choice: 0, // White Paint AZ-93
            radiator_area_m2: 0.05,
            package_heat_capacity: 85.0,
            internal_heat_w: 12.0,
            solder_alloy_choice: 0, // SAC305
            dnp_mm: 8.5,
            standoff_um: 40.0,
            substrate_cte: 3.2,
            underfill_coupling: 0.20,
            dopant_choice: 0, // Phosphorus
            nominal_doping_log: 17.0,
            cryo_temp_k: 4.2,
            cryo_vgs: 1.2,
            sim,
            cached_cooldown,
            cached_orbital_profile,
            cached_freezeout,
            cached_subthreshold,
            cached_id_vds,
        }
    }
}

impl ThermalVacuumDialog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fast non-blocking constructor guaranteeing sub-millisecond initialization for cold boot.
    pub fn new_fast() -> Self {
        Self::default()
    }

    /// Recompute all co-simulation states and refresh cached plot vectors.
    pub fn recompute_sim(&mut self) {
        // Map mission
        self.sim.orbital.mission = match self.mission_choice {
            1 => OrbitalMissionKind::GeostationaryOrbit,
            2 => OrbitalMissionKind::LunarSurface,
            3 => OrbitalMissionKind::DeepSpaceCruise,
            _ => OrbitalMissionKind::LowEarthOrbit,
        };

        // Map coating
        let coating = match self.coating_choice {
            1 => SurfaceCoatingKind::PolishedGold,
            2 => SurfaceCoatingKind::BlackAnodize,
            3 => SurfaceCoatingKind::MultiLayerInsulation,
            4 => SurfaceCoatingKind::BareSilicon,
            _ => SurfaceCoatingKind::WhiteThermalPaint,
        };
        self.sim.radiation.coating = coating;
        self.sim.radiation.area_m2 = self.radiator_area_m2.max(0.001);
        self.sim.orbital.radiation = self.sim.radiation.clone();
        self.sim.orbital.package_heat_capacity_j_per_k = self.package_heat_capacity.max(1.0);
        self.sim.orbital.active_power_dissipation_w = self.internal_heat_w.max(0.0);

        // Map micro-bump
        self.sim.orbital.micro_bump.alloy = match self.solder_alloy_choice {
            1 => SolderAlloyKind::Sn63Pb37,
            2 => SolderAlloyKind::PureIndium,
            _ => SolderAlloyKind::SAC305,
        };
        self.sim.orbital.micro_bump.distance_to_neutral_point_mm = self.dnp_mm.max(0.5);
        self.sim.orbital.micro_bump.bump_height_um = self.standoff_um.max(5.0);
        self.sim.orbital.micro_bump.substrate_cte_ppm_per_k = self.substrate_cte.max(0.5);
        self.sim.orbital.micro_bump.underfill_coupling_factor = self.underfill_coupling.clamp(0.02, 1.0);

        // Map cryogenic dopant
        self.sim.freezeout.dopant = match self.dopant_choice {
            1 => CryogenicDopantKind::BoronInSilicon,
            2 => CryogenicDopantKind::ArsenicInSilicon,
            3 => CryogenicDopantKind::AntimonyInSilicon,
            4 => CryogenicDopantKind::IndiumInSilicon,
            _ => CryogenicDopantKind::PhosphorusInSilicon,
        };
        self.sim.freezeout.nominal_doping_cm3 = 10.0_f64.powf(self.nominal_doping_log);
        self.sim.kink.temp_k = self.cryo_temp_k.max(1.0);

        // Run recompute
        self.sim.recompute();

        // Refresh plot curves
        let mut dark_rad = self.sim.radiation.clone();
        dark_rad.solar_flux_w_m2 = 0.0;
        dark_rad.albedo_flux_w_m2 = 0.0;
        let cooldown_raw = dark_rad.simulate_vacuum_cooldown(
            350.0,
            self.package_heat_capacity,
            self.internal_heat_w,
            3600.0,
            60,
        );
        self.cached_cooldown = cooldown_raw.into_iter().map(|(t, c)| [t, c]).collect();

        let orbital_raw = self.sim.orbital.simulate_orbital_profile(75);
        self.cached_orbital_profile = orbital_raw.into_iter().map(|(t, c)| [t, c]).collect();

        let freezeout_raw = self.sim.freezeout.simulate_freezeout_curve(4.2, 50);
        self.cached_freezeout = freezeout_raw.into_iter().map(|(t, eta)| [t, eta]).collect();

        let ss_raw = self.sim.subthreshold.simulate_subthreshold_swing_curve(50);
        self.cached_subthreshold = ss_raw.into_iter().map(|(t, s)| [t, s]).collect();

        let iv_raw = self.sim.kink.simulate_id_vds_curve(self.cryo_vgs, 2.5, 50);
        self.cached_id_vds = iv_raw.into_iter().map(|(v, i)| [v, i]).collect();
    }

    /// Render modal UI window (standard alias).
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Render modal window for Thermal-Vacuum Co-Simulation.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new("Aerospace Thermal-Vacuum & Orbital Cycling Co-Simulator")
            .open(&mut is_open)
            .default_size(vec2(920.0, 680.0))
            .min_size(vec2(780.0, 560.0))
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    pub fn render_content(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
                    if ui
                        .selectable_label(
                            self.active_tab == ThermalVacuumTab::OrbitalEnvironment,
                            "Orbital Environment",
                        )
                        .clicked()
                    {
                        self.active_tab = ThermalVacuumTab::OrbitalEnvironment;
                    }
                    if ui
                        .selectable_label(
                            self.active_tab == ThermalVacuumTab::StefanBoltzmannCooldown,
                            "Vacuum Stefan-Boltzmann",
                        )
                        .clicked()
                    {
                        self.active_tab = ThermalVacuumTab::StefanBoltzmannCooldown;
                    }
                    if ui
                        .selectable_label(
                            self.active_tab == ThermalVacuumTab::OrbitalCyclingProfile,
                            "Orbital Cycling T(t)",
                        )
                        .clicked()
                    {
                        self.active_tab = ThermalVacuumTab::OrbitalCyclingProfile;
                    }
                    if ui
                        .selectable_label(
                            self.active_tab == ThermalVacuumTab::MicroBumpFatigue,
                            "Micro-Bump Creep-Fatigue",
                        )
                        .clicked()
                    {
                        self.active_tab = ThermalVacuumTab::MicroBumpFatigue;
                    }
                    if ui
                        .selectable_label(
                            self.active_tab == ThermalVacuumTab::CryogenicFreezeoutKink,
                            "Cryo Freeze-Out & Kink TCAD",
                        )
                        .clicked()
                    {
                        self.active_tab = ThermalVacuumTab::CryogenicFreezeoutKink;
                    }
                });

                ui.separator();

                match self.active_tab {
                    ThermalVacuumTab::OrbitalEnvironment => self.render_orbital_env_tab(ui),
                    ThermalVacuumTab::StefanBoltzmannCooldown => self.render_stefan_boltzmann_tab(ui),
                    ThermalVacuumTab::OrbitalCyclingProfile => self.render_orbital_cycling_tab(ui),
                    ThermalVacuumTab::MicroBumpFatigue => self.render_micro_bump_tab(ui),
                    ThermalVacuumTab::CryogenicFreezeoutKink => self.render_cryogenic_freezeout_tab(ui),
                }

                ui.separator();
                self.render_telemetry_footer(ui);
    }

    fn render_orbital_env_tab(&mut self, ui: &mut Ui) {
        ui.heading("Orbital Environment, Solar Radiation & Eclipse Dynamics");
        ui.label(
            "Configure satellite orbit, thermal control coating finish, and incident solar/albedo radiative boundaries.",
        );

        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label("Mission Orbit:");
            let old_m = self.mission_choice;
            egui::ComboBox::from_id_salt("mission_select")
                .selected_text(match self.mission_choice {
                    1 => "Geostationary Earth Orbit (GEO)",
                    2 => "Lunar Surface Mission",
                    3 => "Deep-Space Cruise",
                    _ => "Low Earth Orbit (LEO, 92.5 min)",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.mission_choice, 0, "Low Earth Orbit (LEO, 92.5 min)");
                    ui.selectable_value(&mut self.mission_choice, 1, "Geostationary Earth Orbit (GEO)");
                    ui.selectable_value(&mut self.mission_choice, 2, "Lunar Surface Mission");
                    ui.selectable_value(&mut self.mission_choice, 3, "Deep-Space Cruise");
                });

            ui.add_space(12.0);

            ui.label("Radiator Finish:");
            let old_c = self.coating_choice;
            egui::ComboBox::from_id_salt("coating_select")
                .selected_text(match self.coating_choice {
                    1 => "Polished Gold / Foil (eps ~ 0.04)",
                    2 => "Black Anodize (eps ~ 0.96)",
                    3 => "Multi-Layer Insulation (eps ~ 0.02)",
                    4 => "Bare Silicon Die (eps ~ 0.65)",
                    _ => "White Thermal Paint AZ-93 (eps ~ 0.90)",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.coating_choice, 0, "White Thermal Paint AZ-93 (eps ~ 0.90)");
                    ui.selectable_value(&mut self.coating_choice, 1, "Polished Gold / Foil (eps ~ 0.04)");
                    ui.selectable_value(&mut self.coating_choice, 2, "Black Anodize (eps ~ 0.96)");
                    ui.selectable_value(&mut self.coating_choice, 3, "Multi-Layer Insulation (eps ~ 0.02)");
                    ui.selectable_value(&mut self.coating_choice, 4, "Bare Silicon Die (eps ~ 0.65)");
                });

            if old_m != self.mission_choice || old_c != self.coating_choice {
                self.recompute_sim();
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let mut changed = false;
            ui.label("Radiator Area (m^2):");
            if ui.add(egui::DragValue::new(&mut self.radiator_area_m2).range(0.005..=1.0).speed(0.005)).changed() {
                changed = true;
            }

            ui.add_space(8.0);
            ui.label("Internal Power (W):");
            if ui.add(egui::DragValue::new(&mut self.internal_heat_w).range(0.0..=200.0).speed(0.5)).changed() {
                changed = true;
            }

            ui.add_space(8.0);
            ui.label("Package C_th (J/K):");
            if ui.add(egui::DragValue::new(&mut self.package_heat_capacity).range(5.0..=500.0).speed(1.0)).changed() {
                changed = true;
            }

            if changed {
                self.recompute_sim();
            }
        });

        ui.add_space(8.0);

        // 2D Canvas rendering orbital configuration
        let (response, painter) = ui.allocate_painter(vec2(ui.available_width(), 320.0), Sense::hover());
        let rect = response.rect;

        // Space black background
        painter.rect_filled(rect, 4.0, Color32::from_rgb(8, 10, 18));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(40, 50, 75)), StrokeKind::Outside);

        let center = rect.center();

        // Draw central body (Earth or Moon)
        let body_radius = 45.0;
        let is_lunar = self.mission_choice == 2;
        let body_color = if is_lunar {
            Color32::from_rgb(160, 160, 170)
        } else {
            Color32::from_rgb(50, 100, 180)
        };
        painter.circle_filled(center, body_radius, body_color);

        // Draw atmosphere halo if Earth
        if !is_lunar {
            painter.circle_stroke(center, body_radius + 4.0, Stroke::new(2.5, Color32::from_rgba_unmultiplied(100, 180, 255, 120)));
        }

        // Draw orbital trajectory ring
        let orbit_rx = 180.0;
        let orbit_ry = 95.0;
        let n_pts = 64;
        for i in 0..n_pts {
            let th1 = i as f64 * 2.0 * std::f64::consts::PI / n_pts as f64;
            let th2 = (i + 1) as f64 * 2.0 * std::f64::consts::PI / n_pts as f64;
            let p1 = pos2(center.x + (orbit_rx * th1.cos()) as f32, center.y + (orbit_ry * th1.sin()) as f32);
            let p2 = pos2(center.x + (orbit_rx * th2.cos()) as f32, center.y + (orbit_ry * th2.sin()) as f32);
            painter.line_segment([p1, p2], Stroke::new(1.2, Color32::from_rgba_unmultiplied(120, 150, 200, 160)));
        }

        // Draw solar rays (Sun on left side)
        for y_off in [-80.0, -40.0, 0.0, 40.0, 80.0] {
            let start = pos2(rect.left() + 15.0, center.y + y_off);
            let end = pos2(center.x - 70.0, center.y + y_off);
            painter.line_segment([start, end], Stroke::new(1.5, Color32::from_rgba_unmultiplied(255, 220, 80, 160)));
        }
        painter.text(
            pos2(rect.left() + 25.0, center.y - 100.0),
            egui::Align2::LEFT_CENTER,
            "Direct Solar Irradiance (1361 W/m^2)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(255, 230, 100),
        );

        // Draw eclipse shadow cone (right side)
        let shadow_tip_x = rect.right() - 20.0;
        painter.line_segment([pos2(center.x, center.y - body_radius), pos2(shadow_tip_x, center.y - 75.0)], Stroke::new(1.0, Color32::from_rgba_unmultiplied(80, 80, 110, 180)));
        painter.line_segment([pos2(center.x, center.y + body_radius), pos2(shadow_tip_x, center.y + 75.0)], Stroke::new(1.0, Color32::from_rgba_unmultiplied(80, 80, 110, 180)));
        painter.text(
            pos2(center.x + 90.0, center.y),
            egui::Align2::CENTER_CENTER,
            "Umbra / Eclipse Cone",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(140, 140, 180),
        );

        // Draw Spacecraft position along orbit
        let sat_pos = pos2(center.x - (orbit_rx * 0.7) as f32, center.y - (orbit_ry * 0.7) as f32);
        painter.rect_filled(
            egui::Rect::from_center_size(sat_pos, vec2(16.0, 12.0)),
            2.0,
            Color32::from_rgb(220, 220, 230),
        );
        // Radiator panel wings
        painter.rect_filled(
            egui::Rect::from_center_size(pos2(sat_pos.x - 14.0, sat_pos.y), vec2(10.0, 18.0)),
            1.0,
            Color32::from_rgb(60, 140, 240),
        );
        painter.rect_filled(
            egui::Rect::from_center_size(pos2(sat_pos.x + 14.0, sat_pos.y), vec2(10.0, 18.0)),
            1.0,
            Color32::from_rgb(60, 140, 240),
        );

        painter.text(
            pos2(sat_pos.x, sat_pos.y - 18.0),
            egui::Align2::CENTER_CENTER,
            "Avionics Package / Radiator",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(120, 230, 255),
        );
    }

    fn render_stefan_boltzmann_tab(&mut self, ui: &mut Ui) {
        ui.heading("Stefan-Boltzmann Radiative Dissipation in Vacuum (h_conv = 0)");
        ui.label(
            "Non-linear T^4 radiation cooling to deep space (2.7K) comparing multi-material coatings under zero convection.",
        );

        ui.add_space(8.0);

        let report = self.sim.report().clone();

        ui.columns(2, |cols| {
            // Column 1: Cooldown curve
            cols[0].vertical(|ui| {
                ui.label(RichText::new("Transient Vacuum Cooldown Trajectory T(t)").strong());

                let pts = PlotPoints::new(self.cached_cooldown.iter().map(|p| [p[0], p[1]]).collect());
                let line = Line::new("Radiator Cooldown (deg C)", pts)
                    .color(Color32::from_rgb(80, 200, 255))
                    .width(2.5);

                let zero_c_line = VLine::new("Freezing 0 deg C", 0.0)
                    .color(Color32::from_rgba_unmultiplied(200, 200, 200, 100));

                Plot::new("vacuum_cooldown_plot")
                    .height(290.0)
                    .legend(Legend::default())
                    .x_axis_label("Time in Eclipse (seconds)")
                    .y_axis_label("Surface Temperature (deg C)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(line);
                        plot_ui.vline(zero_c_line);
                    });
            });

            // Column 2: Coating parameters & equilibrium metrics
            cols[1].vertical(|ui| {
                ui.label(RichText::new("Radiative Coating Optical Properties").strong());

                egui::Grid::new("coating_metrics_grid")
                    .num_columns(2)
                    .spacing([20.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Selected Finish:");
                        ui.label(RichText::new(&report.radiator_coating_label).strong());
                        ui.end_row();

                        ui.label("Infrared Emissivity (eps):");
                        ui.label(format!("{:.2}", self.sim.radiation.coating.emissivity()));
                        ui.end_row();

                        ui.label("Solar Absorptance (alpha_s):");
                        ui.label(format!("{:.2}", self.sim.radiation.coating.solar_absorptance()));
                        ui.end_row();

                        ui.label("Solar / IR Ratio (alpha_s / eps):");
                        ui.label(format!("{:.2}", self.sim.radiation.coating.alpha_over_eps_ratio()));
                        ui.end_row();

                        ui.label("Sunlit Equilibrium Temp:");
                        ui.label(
                            RichText::new(format!("{:.1} deg C", report.sunlit_equilibrium_temp_c))
                                .color(Color32::from_rgb(255, 180, 80))
                                .strong(),
                        );
                        ui.end_row();

                        ui.label("Eclipse Equilibrium Temp:");
                        ui.label(
                            RichText::new(format!("{:.1} deg C", report.eclipse_equilibrium_temp_c))
                                .color(Color32::from_rgb(120, 200, 255))
                                .strong(),
                        );
                        ui.end_row();

                        ui.label("Deep-Space Sink Temp:");
                        ui.label("2.725 K (-270.4 deg C)");
                        ui.end_row();
                    });
            });
        });
    }

    fn render_orbital_cycling_tab(&mut self, ui: &mut Ui) {
        ui.heading("Orbital Rapid Thermal Cycling Profile T(t)");
        ui.label(
            "Transient temperature swing during solar illumination and planetary eclipse passes across 1 complete orbit.",
        );

        ui.add_space(8.0);

        let report = self.sim.report().clone();

        ui.horizontal(|ui| {
            ui.label(format!("Orbit: {}", report.mission_label));
            ui.add_space(20.0);
            ui.label(
                RichText::new(format!("Cyclic Swing Delta T: {:.1} K", report.orbital_temp_swing_k))
                    .color(Color32::from_rgb(255, 140, 100))
                    .strong(),
            );
        });

        ui.add_space(6.0);

        let pts = PlotPoints::new(self.cached_orbital_profile.iter().map(|p| [p[0], p[1]]).collect());
        let line = Line::new("Package Temperature (deg C)", pts)
            .color(Color32::from_rgb(255, 160, 60))
            .width(2.5);

        Plot::new("orbital_profile_plot")
            .height(310.0)
            .legend(Legend::default())
            .x_axis_label("Orbital Elapsed Time (minutes)")
            .y_axis_label("Package Temperature (deg C)")
            .show(ui, |plot_ui| {
                plot_ui.line(line);
            });
    }

    fn render_micro_bump_tab(&mut self, ui: &mut Ui) {
        ui.heading("Micro-Bump Solder Interconnect Coffin-Manson Creep-Fatigue");
        ui.label(
            "Evaluation of cyclic shear strain Delta gamma_p and package lifetime projections under orbital thermal swing.",
        );

        ui.add_space(8.0);

        let report = self.sim.report().clone();

        // Qualification Banner
        let is_qualified = report.projected_lifetime_years >= 10.0;
        let (banner_text, banner_bg, banner_fg) = if is_qualified {
            (
                format!(
                    "QUALIFIED: Projected Lifetime {:.1} Years (Threshold >= 10.0 Years, Nf = {:.0} cycles)",
                    report.projected_lifetime_years, report.projected_cycles_to_failure
                ),
                Color32::from_rgb(20, 60, 30),
                Color32::from_rgb(120, 255, 140),
            )
        } else {
            (
                format!(
                    "MARGINAL / AT RISK: Projected Lifetime {:.1} Years (Target: 10.0 Years, Nf = {:.0} cycles)",
                    report.projected_lifetime_years, report.projected_cycles_to_failure
                ),
                Color32::from_rgb(70, 30, 20),
                Color32::from_rgb(255, 160, 120),
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
            // Column 1: Package geometry parameters
            cols[0].vertical(|ui| {
                ui.label(RichText::new("Package Architecture & Micro-Bump Geometry").strong());

                ui.horizontal(|ui| {
                    ui.label("Solder Alloy:");
                    let old_a = self.solder_alloy_choice;
                    egui::ComboBox::from_id_salt("alloy_select")
                        .selected_text(match self.solder_alloy_choice {
                            1 => "Sn63Pb37 (Tin-Lead Aerospace)",
                            2 => "Pure Indium (Cryo-CMOS / Superconducting)",
                            _ => "SAC305 (Sn96.5Ag3.0Cu0.5 Lead-Free)",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.solder_alloy_choice, 0, "SAC305 (Sn96.5Ag3.0Cu0.5 Lead-Free)");
                            ui.selectable_value(&mut self.solder_alloy_choice, 1, "Sn63Pb37 (Tin-Lead Aerospace)");
                            ui.selectable_value(&mut self.solder_alloy_choice, 2, "Pure Indium (Cryo-CMOS / Superconducting)");
                        });

                    if old_a != self.solder_alloy_choice {
                        self.recompute_sim();
                    }
                });

                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label("DNP (mm):");
                    if ui.add(egui::DragValue::new(&mut self.dnp_mm).range(1.0..=25.0).speed(0.2)).changed() {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Standoff (um):");
                    if ui.add(egui::DragValue::new(&mut self.standoff_um).range(10.0..=120.0).speed(1.0)).changed() {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Substrate CTE (ppm/K):");
                    if ui.add(egui::DragValue::new(&mut self.substrate_cte).range(1.0..=20.0).speed(0.2)).changed() {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Underfill Relief Factor:");
                    if ui.add(egui::Slider::new(&mut self.underfill_coupling, 0.05..=1.0)).changed() {
                        changed = true;
                    }
                });

                if changed {
                    self.recompute_sim();
                }
            });

            // Column 2: Fatigue metrics
            cols[1].vertical(|ui| {
                ui.label(RichText::new("Coffin-Manson Low-Cycle Fatigue Metrics").strong());

                egui::Grid::new("fatigue_metrics_grid")
                    .num_columns(2)
                    .spacing([20.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("CTE Mismatch Delta CTE:");
                        ui.label(format!("{:.2} ppm / K", self.sim.orbital.micro_bump.cte_mismatch_ppm_per_k()));
                        ui.end_row();

                        ui.label("Cyclic Shear Strain Delta gamma_p:");
                        ui.label(format!("{:.3} %", report.micro_bump_strain_range_pct));
                        ui.end_row();

                        ui.label("Mean Cycles to Failure Nf:");
                        ui.label(
                            RichText::new(format!("{:.0} cycles", report.projected_cycles_to_failure))
                                .color(Color32::from_rgb(140, 230, 180))
                                .strong(),
                        );
                        ui.end_row();

                        ui.label("Projected Mission Lifetime:");
                        ui.label(
                            RichText::new(format!("{:.1} calendar years", report.projected_lifetime_years))
                                .color(Color32::from_rgb(120, 220, 255))
                                .strong(),
                        );
                        ui.end_row();

                        ui.label("Ductility Exponent c:");
                        ui.label(format!("{:.2}", self.sim.orbital.micro_bump.alloy.fatigue_exponent_c()));
                        ui.end_row();
                    });
            });
        });
    }

    fn render_cryogenic_freezeout_tab(&mut self, ui: &mut Ui) {
        ui.heading("Cryogenic Carrier Freeze-Out, Subthreshold Steepening & Kink TCAD");
        ui.label(
            "Deep cryogenic CMOS physics (4.2K to 300K): dopant freeze-out, subthreshold swing S < 20 mV/dec, and impact-ionization kink.",
        );

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("Dopant Species:");
            let old_d = self.dopant_choice;
            egui::ComboBox::from_id_salt("dopant_select")
                .selected_text(match self.dopant_choice {
                    1 => "Boron in Silicon (p-type, 45 meV)",
                    2 => "Arsenic in Silicon (n-type, 54 meV)",
                    3 => "Antimony in Silicon (n-type, 43 meV)",
                    4 => "Indium in Silicon (p-type deep, 160 meV)",
                    _ => "Phosphorus in Silicon (n-type, 45 meV)",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.dopant_choice, 0, "Phosphorus in Silicon (n-type, 45 meV)");
                    ui.selectable_value(&mut self.dopant_choice, 1, "Boron in Silicon (p-type, 45 meV)");
                    ui.selectable_value(&mut self.dopant_choice, 2, "Arsenic in Silicon (n-type, 54 meV)");
                    ui.selectable_value(&mut self.dopant_choice, 3, "Antimony in Silicon (n-type, 43 meV)");
                    ui.selectable_value(&mut self.dopant_choice, 4, "Indium in Silicon (p-type deep, 160 meV)");
                });

            let mut changed = old_d != self.dopant_choice;

            ui.add_space(10.0);
            ui.label("Operating Temp (K):");
            if ui.add(egui::Slider::new(&mut self.cryo_temp_k, 2.0..=300.0)).changed() {
                changed = true;
            }

            ui.add_space(10.0);
            ui.label("Gate V_gs (V):");
            if ui.add(egui::Slider::new(&mut self.cryo_vgs, 0.2..=2.0)).changed() {
                changed = true;
            }

            if changed {
                self.recompute_sim();
            }
        });

        ui.add_space(8.0);

        ui.columns(2, |cols| {
            // Column 1: Freeze-out & subthreshold swing
            cols[0].vertical(|ui| {
                ui.label(RichText::new("Ionized Carrier Fraction eta(T) & Subthreshold Swing S(T)").strong());

                let freeze_pts = PlotPoints::new(self.cached_freezeout.iter().map(|p| [p[0], p[1]]).collect());
                let freeze_line = Line::new("Ionized Fraction n(T)/N_D", freeze_pts)
                    .color(Color32::from_rgb(100, 220, 140))
                    .width(2.0);

                Plot::new("freezeout_plot")
                    .height(280.0)
                    .legend(Legend::default())
                    .x_axis_label("Temperature (Kelvin)")
                    .y_axis_label("Ionized Fraction n / N_D")
                    .show(ui, |plot_ui| {
                        plot_ui.line(freeze_line);
                    });
            });

            // Column 2: Cryogenic Kink IV curve
            cols[1].vertical(|ui| {
                ui.label(RichText::new("Cryogenic MOSFET I_D(V_DS) Output Kink Effect").strong());

                let iv_pts = PlotPoints::new(self.cached_id_vds.iter().map(|p| [p[0], p[1]]).collect());
                let iv_line = Line::new("I_D(V_DS) at Cryo Temp (mA)", iv_pts)
                    .color(Color32::from_rgb(255, 120, 160))
                    .width(2.0);

                let kink_vline = VLine::new("V_kink Onset (1.0 V)", 1.0)
                    .color(Color32::from_rgba_unmultiplied(255, 200, 80, 140));

                Plot::new("cryo_kink_plot")
                    .height(280.0)
                    .legend(Legend::default())
                    .x_axis_label("Drain Voltage V_DS (Volts)")
                    .y_axis_label("Drain Current I_D (mA)")
                    .show(ui, |plot_ui| {
                        plot_ui.line(iv_line);
                        plot_ui.vline(kink_vline);
                    });
            });
        });
    }

    fn render_telemetry_footer(&mut self, ui: &mut Ui) {
        let report = self.sim.report().clone();

        ui.horizontal(|ui| {
            ui.label(RichText::new("TELEMETRY:").strong().color(Color32::from_rgb(180, 200, 240)));
            ui.label(format!("T_sunlit: {:.1}C", report.sunlit_equilibrium_temp_c));
            ui.separator();
            ui.label(format!("T_eclipse: {:.1}C", report.eclipse_equilibrium_temp_c));
            ui.separator();
            ui.label(format!("Delta T: {:.1}K", report.orbital_temp_swing_k));
            ui.separator();
            ui.label(format!("Nf: {:.0}", report.projected_cycles_to_failure));
            ui.separator();
            ui.label(format!("Life: {:.1} yr", report.projected_lifetime_years));
            ui.separator();
            ui.label(format!("S(77K): {:.1} mV/dec", report.subthreshold_swing_77k_mv_per_dec));
            ui.separator();
            ui.label(format!("S(4.2K): {:.1} mV/dec", report.subthreshold_swing_4k_mv_per_dec));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Recompute Co-Simulation").clicked() {
                    self.recompute_sim();
                }

                if ui.button("LEO CubeSat Preset").clicked() {
                    self.mission_choice = 0;
                    self.coating_choice = 0; // White Paint
                    self.package_heat_capacity = 85.0;
                    self.internal_heat_w = 12.0;
                    self.solder_alloy_choice = 0;
                    self.recompute_sim();
                }

                if ui.button("Cryo-CMOS Preset").clicked() {
                    self.mission_choice = 3; // Deep space
                    self.coating_choice = 1; // Gold
                    self.cryo_temp_k = 4.2;
                    self.solder_alloy_choice = 2; // Indium
                    self.recompute_sim();
                }
            });
        });
    }
}
