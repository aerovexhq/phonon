#![deny(unsafe_code)]

//! Interactive Closed-Loop Dynamic Electro-Thermal & Power Throttling CAD Dialog.
//!
//! Provides a 5-tab CAD electro-thermal co-simulation environment:
//! 1. Dynamic Closed-Loop DVFS Transient Scope (temperature, frequency, and power dynamics under workload step).
//! 2. Self-Consistent Thermal Runaway & Bifurcation (P_gen(T) vs P_diss(T), stability factor, bifurcation limit).
//! 3. Liquid Cold-Plate Microchannel Heat Sink (Re, Nu, convective h, caloric rise, pressure drop, and R_cp).
//! 4. Two-Phase Dielectric Immersion Boiling (Rohsenow pool boiling curve, CHF Zuber limit, and safety margin).
//! 5. TIM-1/TIM-2 Pump-Out Aging & Package Reliability (cycle-dependent degradation and thermal stackup).

use egui::{Color32, Context, Pos2, Rect, RichText, Stroke, StrokeKind, Ui, Vec2, Window};
use egui_plot::{HLine, Legend, Line, Plot, PlotPoints, VLine};
use phonon_solver::electrothermal_throttling::{
    CoolantFluid, ElectrothermalCoSimulator, ImmersionFluidKind,
    ThermalStabilityStatus,
};

/// Active tab in the Electrothermal Throttling Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThrottlingTab {
    DvfsTransientScope,
    ThermalRunawayBifurcation,
    LiquidColdPlate,
    TwoPhaseImmersion,
    TimPumpOutAging,
}

/// Modal dialog for Closed-Loop Dynamic Electro-Thermal & Power Throttling Co-Simulation.
pub struct ElectrothermalThrottlingDialog {
    pub is_open: bool,
    pub active_tab: ThrottlingTab,

    // Core co-simulator
    pub sim: ElectrothermalCoSimulator,

    // Cached plot vectors for instant UI rendering
    pub cached_transient_time_ms: Vec<f64>,
    pub cached_transient_temp_c: Vec<f64>,
    pub cached_transient_freq_ghz: Vec<f64>,
    pub cached_transient_power_w: Vec<f64>,

    pub cached_bifurcation_temps_c: Vec<f64>,
    pub cached_bifurcation_p_gen: Vec<f64>,
    pub cached_bifurcation_p_diss: Vec<f64>,
    pub cached_bifurcation_critical_t: Option<f64>,

    pub cached_boiling_delta_t: Vec<f64>,
    pub cached_boiling_q_flux: Vec<f64>,
    pub cached_boiling_chf: f64,

    pub cached_tim_cycles: Vec<f64>,
    pub cached_tim_r_tot: Vec<f64>,

    // Local UI control state
    pub ui_operating_surface_temp_c: f64,
}

impl Default for ElectrothermalThrottlingDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl ElectrothermalThrottlingDialog {
    /// Instant, non-blocking constructor with sub-millisecond initialization latency.
    pub fn new_fast() -> Self {
        let sim = ElectrothermalCoSimulator::new_fast();

        let cached_transient_time_ms = vec![0.0, 50.0, 100.0, 150.0, 200.0, 300.0, 400.0, 500.0, 750.0, 1000.0];
        let cached_transient_temp_c = vec![45.0, 52.0, 61.0, 68.0, 72.5, 74.0, 74.8, 75.0, 75.0, 75.0];
        let cached_transient_freq_ghz = vec![3.2, 3.2, 3.2, 3.0, 2.8, 2.7, 2.65, 2.65, 2.65, 2.65];
        let cached_transient_power_w = vec![120.0, 280.0, 275.0, 250.0, 230.0, 220.0, 215.0, 215.0, 215.0, 215.0];

        let cached_bifurcation_temps_c = vec![40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0, 110.0, 120.0];
        let cached_bifurcation_p_gen = vec![150.0, 165.0, 185.0, 215.0, 255.0, 310.0, 385.0, 485.0, 620.0];
        let cached_bifurcation_p_diss = vec![80.0, 140.0, 200.0, 260.0, 320.0, 380.0, 440.0, 500.0, 560.0];
        let cached_bifurcation_critical_t = Some(108.5);

        let cached_boiling_delta_t = vec![1.0, 2.0, 5.0, 10.0, 15.0, 20.0, 25.0, 30.0];
        let cached_boiling_q_flux = vec![0.5, 1.2, 4.5, 12.0, 22.0, 28.5, 22.0, 15.0];
        let cached_boiling_chf = 28.5;

        let cached_tim_cycles = vec![0.0, 1000.0, 2000.0, 5000.0, 8000.0, 10000.0, 15000.0];
        let cached_tim_r_tot = vec![0.045, 0.046, 0.048, 0.054, 0.062, 0.068, 0.082];

        Self {
            is_open: false,
            active_tab: ThrottlingTab::DvfsTransientScope,
            sim,
            cached_transient_time_ms,
            cached_transient_temp_c,
            cached_transient_freq_ghz,
            cached_transient_power_w,
            cached_bifurcation_temps_c,
            cached_bifurcation_p_gen,
            cached_bifurcation_p_diss,
            cached_bifurcation_critical_t,
            cached_boiling_delta_t,
            cached_boiling_q_flux,
            cached_boiling_chf,
            cached_tim_cycles,
            cached_tim_r_tot,
            ui_operating_surface_temp_c: 66.0,
        }
    }

    /// Recomputes all cached simulation trajectories.
    pub fn recompute_all(&mut self) {
        let transient = self.sim.run_transient_simulation();
        self.cached_transient_time_ms = transient.time_history_ms;
        self.cached_transient_temp_c = transient.temp_history_c;
        self.cached_transient_freq_ghz = transient.frequency_history_ghz;
        self.cached_transient_power_w = transient.power_history_w;

        let bif = self.sim.generate_bifurcation_curve(35);
        self.cached_bifurcation_temps_c = bif.temperatures_c;
        self.cached_bifurcation_p_gen = bif.heat_generation_w;
        self.cached_bifurcation_p_diss = bif.heat_dissipation_w;
        self.cached_bifurcation_critical_t = bif.bifurcation_temp_c;

        let boiling = self.sim.generate_boiling_curve(25);
        self.cached_boiling_delta_t = boiling.iter().map(|p| p.superheat_delta_t_sat_k).collect();
        self.cached_boiling_q_flux = boiling.iter().map(|p| p.heat_flux_w_cm2).collect();
        self.cached_boiling_chf = phonon_solver::electrothermal_throttling::calculate_zuber_chf(
            &self.sim.immersion.fluid.properties(),
        );

        let tim_aging = self.sim.generate_tim_aging_curve(15000, 25);
        self.cached_tim_cycles = tim_aging.iter().map(|p| p.cycles as f64).collect();
        self.cached_tim_r_tot = tim_aging.iter().map(|p| p.total_tim_resistance_k_w).collect();
    }

    /// Renders the modal window inside the egui context.
    pub fn show(&mut self, ctx: &Context) {
        self.ui(ctx);
    }

    /// Renders the modal window inside the egui context.
    pub fn ui(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new(RichText::new("Dynamic DVFS & Thermal Throttling Co-Simulator").strong())
            .open(&mut is_open)
            .default_size(Vec2::new(960.0, 680.0))
            .min_size(Vec2::new(820.0, 560.0))
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    fn render_content(&mut self, ui: &mut Ui) {
        let telemetry = self.sim.compute_telemetry();

        // Top Status & Telemetry Header
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Electro-Thermal Digital Twin").size(17.0).color(Color32::from_rgb(90, 200, 250)));
            ui.separator();

            let (status_text, status_color) = match telemetry.stability_status {
                ThermalStabilityStatus::Stable => ("STABLE", Color32::from_rgb(70, 210, 110)),
                ThermalStabilityStatus::Marginal => ("MARGINAL", Color32::from_rgb(255, 180, 50)),
                ThermalStabilityStatus::Runaway => ("RUNAWAY DETECTED", Color32::from_rgb(255, 75, 75)),
            };

            ui.label(RichText::new(format!("Stability: {}", status_text)).color(status_color).strong());
            ui.separator();
            ui.label(RichText::new(format!("T_j: {:.1} C", telemetry.junction_temperature_c)).color(Color32::WHITE).strong());
            ui.separator();
            ui.label(RichText::new(format!("P_tot: {:.1} W ({:.1}% Leak)", telemetry.total_power_w, telemetry.leakage_fraction_pct)).color(Color32::from_rgb(220, 220, 120)));
            ui.separator();
            ui.label(RichText::new(format!("Freq: {:.2} GHz", telemetry.current_frequency_ghz)).color(Color32::from_rgb(140, 200, 255)));
            ui.separator();
            ui.label(RichText::new(format!("R_th: {:.3} K/W", telemetry.total_thermal_resistance_k_w)).color(Color32::LIGHT_GRAY));
        });

        ui.add_space(6.0);

        // Tab Bar
        ui.horizontal(|ui| {
            let tabs = [
                (ThrottlingTab::DvfsTransientScope, "1. DVFS Transient Scope"),
                (ThrottlingTab::ThermalRunawayBifurcation, "2. Thermal Runaway Bifurcation"),
                (ThrottlingTab::LiquidColdPlate, "3. Liquid Cold-Plate"),
                (ThrottlingTab::TwoPhaseImmersion, "4. Two-Phase Immersion"),
                (ThrottlingTab::TimPumpOutAging, "5. TIM Pump-Out & Reliability"),
            ];

            for (tab, title) in tabs {
                let is_active = self.active_tab == tab;
                if ui.selectable_label(is_active, RichText::new(title).strong()).clicked() {
                    self.active_tab = tab;
                }
            }
        });

        ui.separator();
        ui.add_space(4.0);

        // Tab Content
        match self.active_tab {
            ThrottlingTab::DvfsTransientScope => self.render_dvfs_transient_tab(ui, &telemetry),
            ThrottlingTab::ThermalRunawayBifurcation => self.render_thermal_runaway_tab(ui, &telemetry),
            ThrottlingTab::LiquidColdPlate => self.render_liquid_coldplate_tab(ui, &telemetry),
            ThrottlingTab::TwoPhaseImmersion => self.render_two_phase_immersion_tab(ui, &telemetry),
            ThrottlingTab::TimPumpOutAging => self.render_tim_aging_tab(ui, &telemetry),
        }
    }

    fn render_dvfs_transient_tab(&mut self, ui: &mut Ui, telemetry: &phonon_solver::electrothermal_throttling::ThrottlingTelemetryReport) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Closed-Loop Hardware Thermal Throttling & DVFS Response").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(RichText::new("Re-Simulate Transient").color(Color32::from_rgb(100, 220, 255))).clicked() {
                    self.recompute_all();
                }
            });
        });

        ui.add_space(4.0);

        // Transient egui_plot
        let plot_height = 280.0;
        let temp_points: Vec<[f64; 2]> = self
            .cached_transient_time_ms
            .iter()
            .zip(self.cached_transient_temp_c.iter())
            .map(|(&t, &temp)| [t, temp])
            .collect();

        let freq_points: Vec<[f64; 2]> = self
            .cached_transient_time_ms
            .iter()
            .zip(self.cached_transient_freq_ghz.iter())
            .map(|(&t, &f)| [t, f * 20.0]) // Scaled for co-plotting (e.g. 4.0 GHz -> 80.0)
            .collect();

        let power_points: Vec<[f64; 2]> = self
            .cached_transient_time_ms
            .iter()
            .zip(self.cached_transient_power_w.iter())
            .map(|(&t, &p)| [t, p * 0.5]) // Scaled
            .collect();

        Plot::new("dvfs_transient_plot")
            .height(plot_height)
            .legend(Legend::default())
            .x_axis_label("Time (ms)")
            .y_axis_label("Junction Temperature (C) / Frequency (x20)")
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("Junction Temp (C)", PlotPoints::from(temp_points))
                        .color(Color32::from_rgb(255, 100, 100))
                        .width(2.5),
                );
                plot_ui.line(
                    Line::new("Clock Freq (GHz x20)", PlotPoints::from(freq_points))
                        .color(Color32::from_rgb(100, 180, 255))
                        .width(2.0),
                );
                plot_ui.line(
                    Line::new("Total Power (W x0.5)", PlotPoints::from(power_points))
                        .color(Color32::from_rgb(255, 215, 0))
                        .width(1.5),
                );

                // Target and critical temperature lines
                plot_ui.hline(
                    HLine::new("T_target (Regulation)", self.sim.dvfs_config.target_temp_c)
                        .color(Color32::from_rgb(80, 220, 120)),
                );
                plot_ui.hline(
                    HLine::new("T_critical (Clock Gating)", self.sim.dvfs_config.critical_temp_c)
                        .color(Color32::from_rgb(255, 80, 80)),
                );
            });

        ui.add_space(6.0);

        // Control Sliders Panel
        ui.columns(2, |cols| {
            cols[0].group(|ui| {
                ui.label(RichText::new("DVFS Closed-Loop Regulator").strong());
                let mut changed = false;

                changed |= ui.add(
                    egui::Slider::new(&mut self.sim.dvfs_config.target_temp_c, 50.0..=95.0)
                        .text("Target Temp (C)"),
                ).changed();

                changed |= ui.add(
                    egui::Slider::new(&mut self.sim.dvfs_config.kp, 0.01..=0.20)
                        .text("Proportional Gain Kp (GHz/K)"),
                ).changed();

                changed |= ui.add(
                    egui::Slider::new(&mut self.sim.dvfs_config.ki, 0.001..=0.05)
                        .text("Integral Gain Ki (GHz/(K*s))"),
                ).changed();

                changed |= ui.add(
                    egui::Slider::new(&mut self.sim.ambient_temp_c, 10.0..=65.0)
                        .text("Ambient Temp (C)"),
                ).changed();

                if changed {
                    self.recompute_all();
                }
            });

            cols[1].group(|ui| {
                ui.label(RichText::new("Telemetry & Throttling Status").strong());
                ui.label(format!("Active P-State: {}", telemetry.active_p_state_name));
                ui.label(format!("Regulated Frequency: {:.2} GHz", telemetry.current_frequency_ghz));
                ui.label(format!("Regulated Voltage: {:.3} V", telemetry.current_voltage_v));
                ui.label(format!("Dynamic Power: {:.1} W", telemetry.dynamic_power_w));
                ui.label(format!("Leakage Power: {:.1} W ({:.1}%)", telemetry.leakage_power_w, telemetry.leakage_fraction_pct));
                ui.label(format!("Thermal Runaway Margin: {:.2}", telemetry.stability_factor));
                if telemetry.is_throttling_active {
                    ui.label(RichText::new("PI Thermal Throttling: ACTIVE").color(Color32::from_rgb(255, 180, 50)).strong());
                } else {
                    ui.label(RichText::new("PI Thermal Throttling: INACTIVE (Full Turbo)").color(Color32::from_rgb(80, 220, 120)));
                }
            });
        });
    }

    fn render_thermal_runaway_tab(&mut self, ui: &mut Ui, telemetry: &phonon_solver::electrothermal_throttling::ThrottlingTelemetryReport) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Self-Consistent Thermal Runaway Bifurcation Analysis").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(t_crit) = self.cached_bifurcation_critical_t {
                    ui.label(RichText::new(format!("Critical Bifurcation Limit: {:.1} C", t_crit)).color(Color32::from_rgb(255, 80, 80)).strong());
                } else {
                    ui.label(RichText::new("Bifurcation Margin: Safe (> 120 C)").color(Color32::from_rgb(80, 220, 120)).strong());
                }
            });
        });

        ui.add_space(4.0);

        let p_gen_points: Vec<[f64; 2]> = self
            .cached_bifurcation_temps_c
            .iter()
            .zip(self.cached_bifurcation_p_gen.iter())
            .map(|(&t, &p)| [t, p])
            .collect();

        let p_diss_points: Vec<[f64; 2]> = self
            .cached_bifurcation_temps_c
            .iter()
            .zip(self.cached_bifurcation_p_diss.iter())
            .map(|(&t, &p)| [t, p])
            .collect();

        Plot::new("bifurcation_plot")
            .height(290.0)
            .legend(Legend::default())
            .x_axis_label("Junction Temperature (C)")
            .y_axis_label("Thermal Power (Watts)")
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("Heat Generation P_gen(T) [Non-linear Leakage]", PlotPoints::from(p_gen_points))
                        .color(Color32::from_rgb(255, 110, 80))
                        .width(2.5),
                );
                plot_ui.line(
                    Line::new("Heat Removal P_diss(T) [(T - T_amb)/R_th]", PlotPoints::from(p_diss_points))
                        .color(Color32::from_rgb(80, 180, 255))
                        .width(2.0),
                );

                // Mark current equilibrium junction temperature
                plot_ui.vline(
                    VLine::new("Equilibrium T_j", telemetry.junction_temperature_c)
                        .color(Color32::from_rgb(255, 230, 80)),
                );

                if let Some(t_crit) = self.cached_bifurcation_critical_t {
                    plot_ui.vline(
                        VLine::new("Runaway Bifurcation T_crit", t_crit)
                            .color(Color32::RED),
                    );
                }
            });

        ui.add_space(6.0);

        ui.columns(2, |cols| {
            cols[0].group(|ui| {
                ui.label(RichText::new("Leakage Model Sensitivity").strong());
                let mut changed = false;

                changed |= ui.add(
                    egui::Slider::new(&mut self.sim.leak_params.i_sub0, 1.0..=25.0)
                        .text("Base Subthreshold Current I_sub0 (A)"),
                ).changed();

                changed |= ui.add(
                    egui::Slider::new(&mut self.sim.leak_params.alpha_vth, 0.0005..=0.0025)
                        .text("V_th Temp Coefficient (V/K)"),
                ).changed();

                changed |= ui.add(
                    egui::Slider::new(&mut self.sim.spreader_resistance_k_w, 0.01..=0.20)
                        .text("Spreader Resistance (K/W)"),
                ).changed();

                if changed {
                    self.recompute_all();
                }
            });

            cols[1].group(|ui| {
                ui.label(RichText::new("Bifurcation Condition Physics").strong());
                ui.label("Self-consistent thermal equilibrium exists when:");
                ui.label(RichText::new("P_gen(T) = P_diss(T)  ==>  T - T_amb - R_th * P_tot(T) = 0").monospace());
                ui.label("Thermal Runaway Bifurcation occurs when:");
                ui.label(RichText::new("d(P_leak)/dT >= 1 / R_th  ==>  Stability Factor S <= 0").monospace().color(Color32::from_rgb(255, 140, 140)));
                ui.separator();
                ui.label(format!("Current Stability Factor S: {:.4}", telemetry.stability_factor));
                ui.label(format!("Heat Removal Capacity 1/R_th: {:.2} W/K", 1.0 / telemetry.total_thermal_resistance_k_w.max(0.001)));
            });
        });
    }

    fn render_liquid_coldplate_tab(&mut self, ui: &mut Ui, _telemetry: &phonon_solver::electrothermal_throttling::ThrottlingTelemetryReport) {
        let perf = self.sim.compute_microchannel_performance();

        ui.horizontal(|ui| {
            ui.label(RichText::new("Liquid Cold-Plate Microchannel Convection & Caloric Model").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("Cold-Plate R_th: {:.4} K/W", perf.total_coldplate_resistance_k_w)).color(Color32::from_rgb(80, 220, 120)).strong());
            });
        });

        ui.add_space(4.0);

        // 2D Microchannel schematic canvas
        let (rect, _response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 160.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);

        painter.rect_filled(rect, 4.0, Color32::from_rgb(20, 26, 36));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(50, 65, 90)), StrokeKind::Middle);

        // Draw microchannels and copper fins
        let num_disp_fins = 14;
        let fin_pitch = rect.width() / (num_disp_fins as f64 * 2.0) as f32;
        let base_y = rect.max.y - 25.0;
        let top_y = rect.min.y + 35.0;

        // Copper base plate
        let base_rect = Rect::from_min_max(Pos2::new(rect.min.x + 8.0, base_y), Pos2::new(rect.max.x - 8.0, rect.max.y - 8.0));
        painter.rect_filled(base_rect, 2.0, Color32::from_rgb(180, 100, 45));

        // Draw fins and fluid channels
        for i in 0..num_disp_fins {
            let fin_x = rect.min.x + 16.0 + (i as f32) * fin_pitch * 2.0;
            let fin_rect = Rect::from_min_max(Pos2::new(fin_x, top_y), Pos2::new(fin_x + fin_pitch * 0.8, base_y));
            painter.rect_filled(fin_rect, 1.0, Color32::from_rgb(210, 125, 60));

            // Fluid channel
            let channel_rect = Rect::from_min_max(Pos2::new(fin_x + fin_pitch * 0.8, top_y), Pos2::new(fin_x + fin_pitch * 2.0, base_y));
            painter.rect_filled(channel_rect, 1.0, Color32::from_rgb(40, 95, 160));
        }

        painter.text(
            Pos2::new(rect.min.x + 16.0, rect.min.y + 12.0),
            egui::Align2::LEFT_TOP,
            format!("Coolant Flow: {:.2} L/min | Microchannel Array ({} Channels, D_h = {:.1} um)", self.sim.microchannel.flow_rate_lpm, perf.channel_count, perf.hydraulic_diameter_m * 1e6),
            egui::FontId::proportional(13.0),
            Color32::from_rgb(150, 215, 255),
        );

        ui.add_space(8.0);

        ui.columns(2, |cols| {
            cols[0].group(|ui| {
                ui.label(RichText::new("Microchannel Geometry & Coolant").strong());
                let mut changed = false;

                ui.horizontal(|ui| {
                    ui.label("Coolant Fluid:");
                    changed |= ui.radio_value(&mut self.sim.microchannel.coolant, CoolantFluid::PureWater, "Pure Water").changed();
                    changed |= ui.radio_value(&mut self.sim.microchannel.coolant, CoolantFluid::PropyleneGlycol50, "50/50 Water-Glycol").changed();
                });

                changed |= ui.add(
                    egui::Slider::new(&mut self.sim.microchannel.flow_rate_lpm, 0.2..=5.0)
                        .text("Flow Rate (L/min)"),
                ).changed();

                let mut w_um = self.sim.microchannel.channel_width_m * 1e6;
                if ui.add(egui::Slider::new(&mut w_um, 50.0..=250.0).text("Channel Width (um)")).changed() {
                    self.sim.microchannel.channel_width_m = w_um * 1e-6;
                    changed = true;
                }

                let mut h_um = self.sim.microchannel.channel_height_m * 1e6;
                if ui.add(egui::Slider::new(&mut h_um, 200.0..=1000.0).text("Channel Height (um)")).changed() {
                    self.sim.microchannel.channel_height_m = h_um * 1e-6;
                    changed = true;
                }

                if changed {
                    self.recompute_all();
                }
            });

            cols[1].group(|ui| {
                ui.label(RichText::new("Hydrodynamic & Thermal Telemetry").strong());
                ui.label(format!("Reynolds Number Re: {:.1} ({})", perf.reynolds_number, if perf.reynolds_number < 2300.0 { "Laminar" } else { "Turbulent" }));
                ui.label(format!("Nusselt Number Nu: {:.2}", perf.nusselt_number));
                ui.label(format!("Convective Coeff h: {:.0} W/(m^2*K)", perf.heat_transfer_coeff_w_m2_k));
                ui.label(format!("Fin Efficiency eta_f: {:.1}%", perf.fin_efficiency * 100.0));
                ui.label(format!("Convective Resistance: {:.4} K/W", perf.convective_resistance_k_w));
                ui.label(format!("Caloric Rise Resistance: {:.4} K/W", perf.caloric_resistance_k_w));
                ui.label(format!("Pressure Drop: {:.2} kPa", perf.pressure_drop_kpa));
                ui.label(format!("Pumping Power: {:.3} W", perf.pumping_power_w));
            });
        });
    }

    fn render_two_phase_immersion_tab(&mut self, ui: &mut Ui, _telemetry: &phonon_solver::electrothermal_throttling::ThrottlingTelemetryReport) {
        let immersion_perf = self.sim.compute_immersion_performance(self.ui_operating_surface_temp_c);

        ui.horizontal(|ui| {
            ui.label(RichText::new("Dielectric Two-Phase Immersion Boiling Curve (Rohsenow & Zuber)").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("CHF Margin: {:.1}%", immersion_perf.chf_margin_percentage)).color(Color32::from_rgb(80, 220, 120)).strong());
            });
        });

        ui.add_space(4.0);

        let curve_points: Vec<[f64; 2]> = self
            .cached_boiling_delta_t
            .iter()
            .zip(self.cached_boiling_q_flux.iter())
            .map(|(&dt, &q)| [dt, q])
            .collect();

        Plot::new("boiling_curve_plot")
            .height(290.0)
            .legend(Legend::default())
            .x_axis_label("Superheat Delta T_sat = T_surf - T_sat (K)")
            .y_axis_label("Heat Flux q'' (W/cm^2)")
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("Rohsenow Nucleate Boiling Curve q''(Delta T_sat)", PlotPoints::from(curve_points))
                        .color(Color32::from_rgb(90, 190, 255))
                        .width(2.5),
                );

                // Critical Heat Flux horizontal line
                plot_ui.hline(
                    HLine::new("Zuber Critical Heat Flux (CHF Limit)", self.cached_boiling_chf)
                        .color(Color32::from_rgb(255, 70, 70)),
                );

                // Operating superheat marker
                let current_delta_t = (self.ui_operating_surface_temp_c - immersion_perf.boiling_point_c).max(0.0);
                plot_ui.vline(
                    VLine::new("Operating Superheat", current_delta_t)
                        .color(Color32::from_rgb(255, 230, 80)),
                );
            });

        ui.add_space(6.0);

        ui.columns(2, |cols| {
            cols[0].group(|ui| {
                ui.label(RichText::new("Immersion Fluid & Operating Point").strong());
                let mut changed = false;

                ui.horizontal(|ui| {
                    ui.label("Dielectric Fluid:");
                    changed |= ui.radio_value(&mut self.sim.immersion.fluid, ImmersionFluidKind::FluorinertFC72, "FC-72 (56 C)").changed();
                    changed |= ui.radio_value(&mut self.sim.immersion.fluid, ImmersionFluidKind::Novec7100, "Novec 7100 (61 C)").changed();
                });

                changed |= ui.add(
                    egui::Slider::new(&mut self.ui_operating_surface_temp_c, 45.0..=85.0)
                        .text("Surface Temp T_surf (C)"),
                ).changed();

                if changed {
                    self.recompute_all();
                }
            });

            cols[1].group(|ui| {
                ui.label(RichText::new("Immersion Boiling Telemetry").strong());
                ui.label(format!("Boiling Point T_sat: {:.1} C", immersion_perf.boiling_point_c));
                ui.label(format!("Current Heat Flux: {:.2} W/cm^2", immersion_perf.current_heat_flux_w_cm2));
                ui.label(format!("Zuber CHF Limit: {:.2} W/cm^2", immersion_perf.critical_heat_flux_w_cm2));
                ui.label(format!("Boiling Heat Transfer Coeff: {:.0} W/(m^2*K)", immersion_perf.effective_heat_transfer_coeff_w_m2_k));
                ui.label(format!("Boiling Thermal Resistance: {:.4} K/W", immersion_perf.boiling_thermal_resistance_k_w));
                ui.label(format!("Regime: {}", if immersion_perf.is_nucleate_boiling_active { "Active Nucleate Boiling" } else { "Single-Phase Natural Convection" }));
            });
        });
    }

    fn render_tim_aging_tab(&mut self, ui: &mut Ui, telemetry: &phonon_solver::electrothermal_throttling::ThrottlingTelemetryReport) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("TIM-1/TIM-2 Pump-Out Degradation & Datacenter Lifetime").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("Current TIM R_th: {:.4} K/W", telemetry.tim_thermal_resistance_k_w)).color(Color32::from_rgb(255, 180, 80)).strong());
            });
        });

        ui.add_space(4.0);

        let tim_points: Vec<[f64; 2]> = self
            .cached_tim_cycles
            .iter()
            .zip(self.cached_tim_r_tot.iter())
            .map(|(&c, &r)| [c, r])
            .collect();

        Plot::new("tim_aging_plot")
            .height(280.0)
            .legend(Legend::default())
            .x_axis_label("Cumulative Thermal Power Cycles")
            .y_axis_label("Total TIM Resistance R_TIM (K/W)")
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("TIM Resistance R_TIM(N) [Fickian Pump-Out Model]", PlotPoints::from(tim_points))
                        .color(Color32::from_rgb(255, 160, 60))
                        .width(2.5),
                );

                plot_ui.vline(
                    VLine::new("Current Power Cycles", self.sim.tim_aging.power_cycles as f64)
                        .color(Color32::from_rgb(100, 200, 255)),
                );
            });

        ui.add_space(6.0);

        ui.columns(2, |cols| {
            cols[0].group(|ui| {
                ui.label(RichText::new("Thermal Interface Material Parameters").strong());
                let mut changed = false;

                let mut cycles_f = self.sim.tim_aging.power_cycles as f64;
                if ui.add(egui::Slider::new(&mut cycles_f, 0.0..=20000.0).text("Power Cycles (N)")).changed() {
                    self.sim.tim_aging.power_cycles = cycles_f.round() as usize;
                    changed = true;
                }

                changed |= ui.add(
                    egui::Slider::new(&mut self.sim.tim_aging.initial_blt_um, 15.0..=80.0)
                        .text("Bondline Thickness BLT (um)"),
                ).changed();

                changed |= ui.add(
                    egui::Slider::new(&mut self.sim.tim_aging.thermal_conductivity_w_m_k, 2.0..=12.0)
                        .text("Conductivity k_tim (W/(m*K))"),
                ).changed();

                changed |= ui.add(
                    egui::Slider::new(&mut self.sim.tim_aging.pump_out_rate_coeff, 0.001..=0.010)
                        .text("Pump-Out Rate gamma"),
                ).changed();

                if changed {
                    self.recompute_all();
                }
            });

            cols[1].group(|ui| {
                ui.label(RichText::new("Lifetime & Temperature Impact").strong());
                let r_base = self.sim.tim_aging.baseline_resistance_k_w();
                let r_aged = self.sim.tim_aging.total_resistance_k_w();
                let delta_r = (r_aged - r_base).max(0.0);
                let delta_t_penalty = telemetry.total_power_w * delta_r;

                ui.label(format!("Baseline Unaged R_TIM: {:.4} K/W", r_base));
                ui.label(format!("Current Degraded R_TIM: {:.4} K/W", r_aged));
                ui.label(format!("Degradation Percentage: +{:.1}%", ((r_aged - r_base) / r_base.max(1e-4)) * 100.0));
                ui.label(RichText::new(format!("Junction Temp Penalty: +{:.2} C", delta_t_penalty)).color(Color32::from_rgb(255, 120, 120)).strong());
                ui.separator();
                ui.label("Package Thermal Stackup:");
                ui.label(format!("Die -> TIM-1 ({:.4} K/W) -> Spreader ({:.4} K/W) -> Cold-Plate ({:.4} K/W)", r_aged, self.sim.spreader_resistance_k_w, telemetry.cooling_thermal_resistance_k_w));
            });
        });
    }
}
