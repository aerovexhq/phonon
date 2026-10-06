#![deny(unsafe_code)]

//! Interactive Physics-Based Silicon Aging, Reliability & Electromigration (EM) CAD Dialog.
//!
//! Provides a 5-tab CAD reliability and 10-year datacenter derating environment:
//! 1. Bias Temperature Instability (NBTI / PBTI reaction-diffusion V_th shift & AC recovery).
//! 2. Hot Carrier Injection (HCI substrate current, lateral field & transconductance degradation).
//! 3. Time-Dependent Dielectric Breakdown (TDDB Weibull distribution, area scaling & FIT rate).
//! 4. Interconnect Electromigration (Black's equation across M1-M15, Joule heating & Blech immortality).
//! 5. 10-Year Datacenter Derating & Guardband Synthesis (f_max degradation & V_dd margin compensation).

use egui::{Color32, Context, RichText, Stroke, Ui, Vec2, Window};
use egui_plot::{HLine, Legend, Line, Plot, PlotPoints, VLine};
use phonon_solver::silicon_aging::{
    evaluate_full_metal_stack, MetalLayerEmResult, MetalLayerId, SiliconAgingCoSimulator,
    SiliconAgingTelemetryReport, SiliconAgingTimeCurves,
};

/// Active tab in the Silicon Aging & Reliability Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgingTab {
    BtiDrift,
    HciDegradation,
    TddbReliability,
    InterconnectEm,
    DatacenterDerating,
}

/// Modal dialog for Physics-Based Silicon Aging, Reliability & Electromigration Co-Simulation.
pub struct SiliconAgingDialog {
    pub is_open: bool,
    pub active_tab: AgingTab,

    // Core co-simulator
    pub sim: SiliconAgingCoSimulator,

    // Cached plot data
    pub cached_curves: SiliconAgingTimeCurves,
    pub cached_em_results: Vec<MetalLayerEmResult>,
    pub cached_telemetry: SiliconAgingTelemetryReport,
}

impl Default for SiliconAgingDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl SiliconAgingDialog {
    /// Instant non-blocking constructor ensuring sub-microsecond cold boot latency.
    pub fn new_fast() -> Self {
        let sim = SiliconAgingCoSimulator::new_fast();

        // Pre-seeded baseline vectors (11 points across 0.01 to 15 years)
        let time_years = vec![
            0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 7.5, 10.0, 15.0,
        ];
        let nbti_shifts_mv = vec![
            7.12, 9.31, 10.45, 12.18, 13.67, 15.35, 17.23, 20.08, 21.41, 22.42, 23.98,
        ];
        let pbti_shifts_mv = vec![
            2.69, 3.51, 3.94, 4.59, 5.16, 5.79, 6.50, 7.58, 8.08, 8.46, 9.05,
        ];
        let hci_shifts_mv = vec![
            0.08, 0.17, 0.23, 0.35, 0.49, 0.67, 0.92, 1.39, 1.67, 1.86, 2.24,
        ];
        let total_shifts_mv = vec![
            7.20, 9.48, 10.68, 12.53, 14.16, 16.02, 18.15, 21.47, 23.08, 24.28, 26.22,
        ];
        let freq_ghz = vec![
            3.44, 3.42, 3.41, 3.39, 3.38, 3.36, 3.35, 3.32, 3.31, 3.30, 3.28,
        ];
        let tddb_failure_prob = vec![
            1.2e-7, 1.1e-6, 3.1e-6, 1.2e-5, 3.5e-5, 9.8e-5, 2.4e-4, 5.1e-4, 6.2e-4, 7.0e-4, 8.4e-4,
        ];
        let tddb_weibull_w = vec![
            -15.9, -13.7, -12.7, -11.3, -10.3, -9.2, -8.3, -7.6, -7.4, -7.3, -7.1,
        ];
        let required_guardband_mv = vec![
            8.28, 10.90, 12.28, 14.41, 16.28, 18.42, 20.87, 24.69, 26.54, 27.92, 30.15,
        ];

        let cached_curves = SiliconAgingTimeCurves {
            time_years,
            nbti_shifts_mv,
            pbti_shifts_mv,
            hci_shifts_mv,
            total_shifts_mv,
            freq_ghz,
            tddb_failure_prob,
            tddb_weibull_w,
            required_guardband_mv,
        };

        let cached_em_results = vec![
            MetalLayerEmResult {
                layer_id: MetalLayerId::M1,
                name: "M1 (Local Contact FinFET)".to_string(),
                cross_section_um2: 0.001152,
                current_density_ma_per_um2: 104.17,
                current_density_a_per_cm2: 1.042e7,
                joule_heating_delta_t_k: 0.45,
                effective_metal_temp_k: 358.60,
                jl_product_a_per_cm: 1875.0,
                is_blech_immortal: true,
                mttf_hours: 1.0e8,
                mttf_years: 11407.7,
                resistance_drift_10yr_pct: 0.05,
                is_em_compliant: true,
            },
            MetalLayerEmResult {
                layer_id: MetalLayerId::M2,
                name: "M2 (Local Signal)".to_string(),
                cross_section_um2: 0.001568,
                current_density_ma_per_um2: 102.04,
                current_density_a_per_cm2: 1.020e7,
                joule_heating_delta_t_k: 0.52,
                effective_metal_temp_k: 358.67,
                jl_product_a_per_cm: 2244.0,
                is_blech_immortal: true,
                mttf_hours: 1.0e8,
                mttf_years: 11407.7,
                resistance_drift_10yr_pct: 0.05,
                is_em_compliant: true,
            },
            MetalLayerEmResult {
                layer_id: MetalLayerId::M15,
                name: "M15 (RDL / Bump Metal)".to_string(),
                cross_section_um2: 7.200,
                current_density_ma_per_um2: 36.11,
                current_density_a_per_cm2: 3.611e6,
                joule_heating_delta_t_k: 0.28,
                effective_metal_temp_k: 358.43,
                jl_product_a_per_cm: 79442.0,
                is_blech_immortal: false,
                mttf_hours: 8.5e5,
                mttf_years: 96.9,
                resistance_drift_10yr_pct: 1.03,
                is_em_compliant: true,
            },
        ];

        let cached_telemetry = SiliconAgingTelemetryReport {
            v_dd_nominal_v: 0.85,
            operating_temp_c: 85.0,
            nominal_freq_ghz: 3.50,
            duty_cycle: 0.50,
            ten_year_nbti_shift_mv: 22.42,
            ten_year_pbti_shift_mv: 8.46,
            ten_year_hci_shift_mv: 1.86,
            ten_year_total_shift_mv: 24.28,
            ten_year_freq_penalty_pct: 5.72,
            ten_year_aged_freq_ghz: 3.30,
            tddb_10yr_failure_prob: 0.000704,
            tddb_fit_rate: 8.03,
            min_em_mttf_years: 96.9,
            critical_em_layer: "M15 (RDL / Bump Metal)".to_string(),
            recommended_guardband_mv: 27.92,
            is_10yr_qualified: true,
        };

        Self {
            is_open: false,
            active_tab: AgingTab::BtiDrift,
            sim,
            cached_curves,
            cached_em_results,
            cached_telemetry,
        }
    }

    /// Recomputes all aging degradation curves, EM stack, and telemetry from current parameters.
    pub fn recompute_all(&mut self) {
        self.cached_curves = self.sim.generate_time_curves(30);
        self.cached_em_results =
            evaluate_full_metal_stack(&self.sim.metal_stack, &self.sim.em, self.sim.temp_k());
        self.cached_telemetry = self.sim.generate_telemetry_report();
    }

    /// Main UI entry point rendering the Silicon Aging & Reliability dialog.
    pub fn ui(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new("Silicon Aging, Reliability & Electromigration (EM)")
            .open(&mut is_open)
            .default_size(Vec2::new(960.0, 680.0))
            .min_size(Vec2::new(780.0, 520.0))
            .resizable(true)
            .show(ctx, |ui| {
                self.render_contents(ui);
            });
        self.is_open = is_open;
    }

    /// Alias for `ui` to provide consistent modal presentation ergonomics.
    pub fn show(&mut self, ctx: &Context) {
        self.ui(ctx);
    }

    fn render_contents(&mut self, ui: &mut Ui) {
        // Tab Navigation
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.active_tab, AgingTab::BtiDrift, "1. BTI (NBTI/PBTI)");
            ui.selectable_value(&mut self.active_tab, AgingTab::HciDegradation, "2. Hot Carrier (HCI)");
            ui.selectable_value(&mut self.active_tab, AgingTab::TddbReliability, "3. TDDB Breakdown");
            ui.selectable_value(&mut self.active_tab, AgingTab::InterconnectEm, "4. Interconnect EM (M1-M15)");
            ui.selectable_value(&mut self.active_tab, AgingTab::DatacenterDerating, "5. 10-Yr Datacenter Derating");
        });
        ui.separator();

        match self.active_tab {
            AgingTab::BtiDrift => self.render_bti_tab(ui),
            AgingTab::HciDegradation => self.render_hci_tab(ui),
            AgingTab::TddbReliability => self.render_tddb_tab(ui),
            AgingTab::InterconnectEm => self.render_em_tab(ui),
            AgingTab::DatacenterDerating => self.render_derating_tab(ui),
        }

        ui.separator();
        self.render_telemetry_footer(ui);
    }

    fn render_bti_tab(&mut self, ui: &mut Ui) {
        ui.heading("Bias Temperature Instability (NBTI / PBTI) Reaction-Diffusion Scope");
        ui.label(
            "pMOS NBTI and nMOS PBTI threshold shifts Delta V_th(t) with dynamic AC stress recovery over 15 years.",
        );

        ui.horizontal(|ui| {
            let mut changed = false;
            ui.group(|ui| {
                ui.label(RichText::new("Operating Conditions").strong());
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.sim.v_dd_nominal_v, 0.60..=1.20)
                            .text("V_dd (V)")
                            .step_by(0.01),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.sim.temp_c, 25.0..=125.0)
                            .text("T_die (C)")
                            .step_by(1.0),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.sim.duty_cycle, 0.05..=1.00)
                            .text("Duty Cycle alpha")
                            .step_by(0.05),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.sim.bti.eot_nm, 0.8..=2.0)
                            .text("EOT (nm)")
                            .step_by(0.05),
                    )
                    .changed();

                if ui.button("Run Silicon Aging Simulation").clicked() || changed {
                    self.recompute_all();
                }
            });

            ui.vertical(|ui| {
                ui.label(
                    RichText::new(format!(
                        "10-Year pMOS NBTI Shift: {:.2} mV",
                        self.cached_telemetry.ten_year_nbti_shift_mv
                    ))
                    .color(Color32::from_rgb(255, 120, 120)),
                );
                ui.label(
                    RichText::new(format!(
                        "10-Year nMOS PBTI Shift: {:.2} mV",
                        self.cached_telemetry.ten_year_pbti_shift_mv
                    ))
                    .color(Color32::from_rgb(120, 200, 255)),
                );
                ui.label(
                    RichText::new(format!(
                        "Combined Critical Path V_th Shift: {:.2} mV",
                        self.cached_telemetry.ten_year_total_shift_mv
                    ))
                    .strong()
                    .color(Color32::from_rgb(255, 215, 0)),
                );
                ui.label(format!(
                    "Drive Current Degradation: -{:.2}%",
                    self.cached_telemetry.ten_year_freq_penalty_pct
                ));
            });
        });

        ui.add_space(8.0);

        let pmos_points: Vec<[f64; 2]> = self
            .cached_curves
            .time_years
            .iter()
            .zip(self.cached_curves.nbti_shifts_mv.iter())
            .map(|(&t, &v)| [t, v])
            .collect();

        let nmos_points: Vec<[f64; 2]> = self
            .cached_curves
            .time_years
            .iter()
            .zip(self.cached_curves.pbti_shifts_mv.iter())
            .map(|(&t, &v)| [t, v])
            .collect();

        let total_points: Vec<[f64; 2]> = self
            .cached_curves
            .time_years
            .iter()
            .zip(self.cached_curves.total_shifts_mv.iter())
            .map(|(&t, &v)| [t, v])
            .collect();

        Plot::new("bti_plot")
            .height(280.0)
            .legend(Legend::default())
            .x_axis_label("Operating Time (Years)")
            .y_axis_label("Delta V_th (mV)")
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("pMOS NBTI Shift (mV)", PlotPoints::from(pmos_points))
                        .color(Color32::from_rgb(255, 100, 100))
                        .stroke(Stroke::new(2.5, Color32::from_rgb(255, 100, 100))),
                );
                plot_ui.line(
                    Line::new("nMOS PBTI Shift (mV)", PlotPoints::from(nmos_points))
                        .color(Color32::from_rgb(100, 180, 255))
                        .stroke(Stroke::new(2.0, Color32::from_rgb(100, 180, 255))),
                );
                plot_ui.line(
                    Line::new("Total Critical Path Shift (mV)", PlotPoints::from(total_points))
                        .color(Color32::from_rgb(255, 215, 0))
                        .stroke(Stroke::new(2.0, Color32::from_rgb(255, 215, 0))),
                );
                plot_ui.vline(
                    VLine::new("10-Year Datacenter Goal", 10.0)
                        .color(Color32::from_rgb(120, 255, 120))
                        .stroke(Stroke::new(1.5, Color32::from_rgb(120, 255, 120))),
                );
                plot_ui.hline(
                    HLine::new("Max 85 mV Budget Limit", 85.0)
                        .color(Color32::from_rgb(255, 80, 80))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(255, 80, 80))),
                );
            });
    }

    fn render_hci_tab(&mut self, ui: &mut Ui) {
        ui.heading("Hot Carrier Injection (HCI) Degradation");
        ui.label(
            "Impact ionization substrate current I_sub, lateral drain pinch-off electric fields, and transconductance loss.",
        );

        ui.horizontal(|ui| {
            let mut changed = false;
            ui.group(|ui| {
                ui.label(RichText::new("HCI FinFET Parameters").strong());
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.sim.hci.channel_length_nm, 8.0..=32.0)
                            .text("L_g (nm)")
                            .step_by(1.0),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.sim.hci.pinch_off_length_nm, 5.0..=20.0)
                            .text("l_pinch (nm)")
                            .step_by(0.5),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.sim.hci.phi_i_ev, 1.0..=1.8)
                            .text("phi_i (eV)")
                            .step_by(0.05),
                    )
                    .changed();

                if changed {
                    self.recompute_all();
                }
            });

            ui.vertical(|ui| {
                let e_lat = (self.sim.v_dd_nominal_v - self.sim.hci.v_dsat_v).max(0.01)
                    / self.sim.hci.pinch_off_length_nm;
                ui.label(format!("Lateral Electric Field E_m: {:.3} V/nm", e_lat));
                ui.label(format!(
                    "10-Year HCI V_th Shift: {:.2} mV",
                    self.cached_telemetry.ten_year_hci_shift_mv
                ));
                let delta_gm = self.sim.hci.a_gm
                    * (self.cached_telemetry.ten_year_hci_shift_mv * 1.0e-3)
                    * 100.0;
                ui.label(format!("Transconductance Loss Delta g_m: -{:.2}%", delta_gm));
            });
        });

        ui.add_space(8.0);

        let hci_points: Vec<[f64; 2]> = self
            .cached_curves
            .time_years
            .iter()
            .zip(self.cached_curves.hci_shifts_mv.iter())
            .map(|(&t, &v)| [t, v])
            .collect();

        Plot::new("hci_plot")
            .height(280.0)
            .legend(Legend::default())
            .x_axis_label("Operating Time (Years)")
            .y_axis_label("Delta V_th,HCI (mV)")
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("HCI V_th Shift (mV)", PlotPoints::from(hci_points))
                        .color(Color32::from_rgb(255, 160, 50))
                        .stroke(Stroke::new(2.5, Color32::from_rgb(255, 160, 50))),
                );
                plot_ui.vline(
                    VLine::new("10-Year Horizon", 10.0)
                        .color(Color32::from_rgb(120, 255, 120))
                        .stroke(Stroke::new(1.5, Color32::from_rgb(120, 255, 120))),
                );
            });
    }

    fn render_tddb_tab(&mut self, ui: &mut Ui) {
        ui.heading("Time-Dependent Dielectric Breakdown (TDDB) Reliability");
        ui.label(
            "Gate oxide defect percolation statistics, full-chip Poisson area scaling, and Weibull failure cumulative distribution.",
        );

        ui.horizontal(|ui| {
            let mut changed = false;
            ui.group(|ui| {
                ui.label(RichText::new("Gate Dielectric & Chip Area").strong());
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.sim.tddb.chip_gate_area_mm2, 5.0..=120.0)
                            .text("Chip Area (mm^2)")
                            .step_by(5.0),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.sim.tddb.weibull_beta, 1.1..=2.0)
                            .text("Weibull Slope beta")
                            .step_by(0.05),
                    )
                    .changed();

                if changed {
                    self.recompute_all();
                }
            });

            ui.vertical(|ui| {
                ui.label(
                    RichText::new(format!(
                        "10-Year Cumulative Failure Probability: {:.4}%",
                        self.cached_telemetry.tddb_10yr_failure_prob * 100.0
                    ))
                    .strong()
                    .color(Color32::from_rgb(120, 230, 255)),
                );
                ui.label(format!(
                    "FIT Rate (Failures / 10^9 hours): {:.2} FIT",
                    self.cached_telemetry.tddb_fit_rate
                ));
                let fit_pass = self.cached_telemetry.tddb_fit_rate < 50.0;
                ui.label(
                    RichText::new(if fit_pass {
                        "TDDB Datacenter Target: PASS (< 50 FIT)"
                    } else {
                        "TDDB Datacenter Target: EXCEEDED (> 50 FIT)"
                    })
                    .color(if fit_pass {
                        Color32::from_rgb(80, 220, 100)
                    } else {
                        Color32::from_rgb(255, 80, 80)
                    }),
                );
            });
        });

        ui.add_space(8.0);

        let tddb_points: Vec<[f64; 2]> = self
            .cached_curves
            .time_years
            .iter()
            .zip(self.cached_curves.tddb_failure_prob.iter())
            .map(|(&t, &f)| [t, f * 100.0])
            .collect();

        Plot::new("tddb_plot")
            .height(280.0)
            .legend(Legend::default())
            .x_axis_label("Operating Time (Years)")
            .y_axis_label("Cumulative Failure Probability (%)")
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new(
                        "Full-Chip Dielectric Failure F(t) (%)",
                        PlotPoints::from(tddb_points),
                    )
                    .color(Color32::from_rgb(80, 200, 255))
                    .stroke(Stroke::new(2.5, Color32::from_rgb(80, 200, 255))),
                );
                plot_ui.vline(
                    VLine::new("10-Year Horizon", 10.0)
                        .color(Color32::from_rgb(120, 255, 120))
                        .stroke(Stroke::new(1.5, Color32::from_rgb(120, 255, 120))),
                );
                plot_ui.hline(
                    HLine::new("0.10% Datacenter Limit", 0.10)
                        .color(Color32::from_rgb(255, 100, 100))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(255, 100, 100))),
                );
            });
    }

    fn render_em_tab(&mut self, ui: &mut Ui) {
        ui.heading("Interconnect Electromigration (EM) across Layers M1 to M15");
        ui.label(
            "Black's Equation MTTF, Joule self-heating derating, and Blech length immortality limits (j*L <= 3500 A/cm).",
        );

        ui.horizontal(|ui| {
            ui.label(format!(
                "Critical Interconnect Bottleneck: {}",
                self.cached_telemetry.critical_em_layer
            ));
            ui.label(format!(
                "Minimum MTTF: {:.1} Years",
                self.cached_telemetry.min_em_mttf_years
            ));
        });

        ui.add_space(8.0);

        egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
            egui::Grid::new("em_table")
                .striped(true)
                .min_col_width(80.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("Layer").strong());
                    ui.label(RichText::new("Width (nm)").strong());
                    ui.label(RichText::new("Thick (nm)").strong());
                    ui.label(RichText::new("J (MA/cm^2)").strong());
                    ui.label(RichText::new("Delta T_Joule").strong());
                    ui.label(RichText::new("Blech Product").strong());
                    ui.label(RichText::new("Blech Status").strong());
                    ui.label(RichText::new("MTTF (Years)").strong());
                    ui.label(RichText::new("10-Yr Drift").strong());
                    ui.end_row();

                    for (i, layer) in self.sim.metal_stack.iter().enumerate() {
                        let res = self.cached_em_results.get(i);
                        ui.label(layer.layer_id.short_name());
                        ui.label(format!("{:.0}", layer.width_nm));
                        ui.label(format!("{:.0}", layer.thickness_nm));

                        if let Some(r) = res {
                            let j_ma_cm2 = r.current_density_a_per_cm2 / 1.0e6;
                            ui.label(format!("{:.2}", j_ma_cm2));
                            ui.label(format!("{:.2} K", r.joule_heating_delta_t_k));
                            ui.label(format!("{:.0} A/cm", r.jl_product_a_per_cm));

                            if r.is_blech_immortal {
                                ui.label(
                                    RichText::new("IMMORTAL")
                                        .strong()
                                        .color(Color32::from_rgb(80, 220, 100)),
                                );
                                ui.label("Infinite (>10^4 yr)");
                                ui.label("< 0.1%");
                            } else {
                                ui.label(
                                    RichText::new("FINITE")
                                        .color(Color32::from_rgb(255, 180, 80)),
                                );
                                let mttf_text = if r.mttf_years > 500.0 {
                                    "> 500 yr".to_string()
                                } else {
                                    format!("{:.1} yr", r.mttf_years)
                                };
                                ui.label(mttf_text);
                                ui.label(format!("{:.2}%", r.resistance_drift_10yr_pct));
                            }
                        } else {
                            ui.label("-");
                            ui.label("-");
                            ui.label("-");
                            ui.label("-");
                            ui.label("-");
                            ui.label("-");
                        }
                        ui.end_row();
                    }
                });
        });
    }

    fn render_derating_tab(&mut self, ui: &mut Ui) {
        ui.heading("10-Year Datacenter Derating & Guardbanding Synthesis");
        ui.label(
            "Clock frequency degradation f_max(t) and proactive V_dd margin synthesis over 10-year mission life.",
        );

        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.label(RichText::new("Qualification Sign-Off").strong());
                let qualified = self.cached_telemetry.is_10yr_qualified;
                let (status_text, color) = if qualified {
                    ("PASS: 10-YEAR DATACENTER QUALIFIED", Color32::from_rgb(60, 220, 100))
                } else {
                    ("FAIL: 10-YEAR WEAR-OUT BUDGET EXCEEDED", Color32::from_rgb(255, 60, 60))
                };
                ui.label(RichText::new(status_text).size(16.0).strong().color(color));

                ui.label(format!(
                    "Nominal Clock: {:.2} GHz -> Aged 10-Yr Clock: {:.2} GHz (-{:.2}%)",
                    self.cached_telemetry.nominal_freq_ghz,
                    self.cached_telemetry.ten_year_aged_freq_ghz,
                    self.cached_telemetry.ten_year_freq_penalty_pct
                ));
                ui.label(
                    RichText::new(format!(
                        "Recommended V_dd Guardband: +{:.1} mV",
                        self.cached_telemetry.recommended_guardband_mv
                    ))
                    .strong()
                    .color(Color32::from_rgb(255, 215, 0)),
                );
            });
        });

        ui.add_space(8.0);

        let freq_points: Vec<[f64; 2]> = self
            .cached_curves
            .time_years
            .iter()
            .zip(self.cached_curves.freq_ghz.iter())
            .map(|(&t, &f)| [t, f])
            .collect();

        Plot::new("freq_derating_plot")
            .height(280.0)
            .legend(Legend::default())
            .x_axis_label("Operating Time (Years)")
            .y_axis_label("Maximum Operating Frequency f_max (GHz)")
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new(
                        "Degraded Maximum Frequency f_max(t) (GHz)",
                        PlotPoints::from(freq_points),
                    )
                    .color(Color32::from_rgb(100, 220, 255))
                    .stroke(Stroke::new(2.5, Color32::from_rgb(100, 220, 255))),
                );
                plot_ui.vline(
                    VLine::new("10-Year Horizon", 10.0)
                        .color(Color32::from_rgb(120, 255, 120))
                        .stroke(Stroke::new(1.5, Color32::from_rgb(120, 255, 120))),
                );
                plot_ui.hline(
                    HLine::new(
                        "15% Derating Floor Limit (GHz)",
                        self.cached_telemetry.nominal_freq_ghz * 0.85,
                    )
                    .color(Color32::from_rgb(255, 80, 80))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(255, 80, 80))),
                );
            });
    }

    fn render_telemetry_footer(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(format!("V_dd: {:.2} V", self.cached_telemetry.v_dd_nominal_v));
            ui.label("|");
            ui.label(format!("T_die: {:.1} C", self.cached_telemetry.operating_temp_c));
            ui.label("|");
            ui.label(format!(
                "10-Yr Total Delta V_th: {:.1} mV",
                self.cached_telemetry.ten_year_total_shift_mv
            ));
            ui.label("|");
            ui.label(format!(
                "Aged Frequency: {:.2} GHz",
                self.cached_telemetry.ten_year_aged_freq_ghz
            ));
            ui.label("|");
            ui.label(format!(
                "V_dd Guardband: +{:.1} mV",
                self.cached_telemetry.recommended_guardband_mv
            ));
            ui.label("|");
            let qual = self.cached_telemetry.is_10yr_qualified;
            ui.label(
                RichText::new(if qual { "10-Yr Signoff: PASS" } else { "10-Yr Signoff: FAIL" })
                    .strong()
                    .color(if qual {
                        Color32::from_rgb(80, 220, 100)
                    } else {
                        Color32::from_rgb(255, 80, 80)
                    }),
            );
        });
    }
}
