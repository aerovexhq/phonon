#![deny(unsafe_code)]

//! CAD Dialog for Topological Acoustic Non-Abelian Holonomic Qubit Braiding Co-Processor & Fractional Valley Repeater (Phase 468).
//!
//! Provides interactive CAD simulation for higher-order corner Wilczek-Zee holonomic gates,
//! fractional valley-Chern waveguide routing, and fault-tolerant cryogenic quantum acoustic processing.

use egui::{Color32, Context, RichText, Sense, Stroke, StrokeKind, Ui, Vec2, Window};
use phonon_solver::holonomic_qubit_repeater::{
    FractionalValleyMetrics, FractionalValleyParams, HolonomicBraidingMetrics,
    HolonomicBraidingParams, HolonomicCornerSpatialPoint, HolonomicCryoMetrics,
    HolonomicCryoParams, HolonomicLoopPoint, HolonomicQubitGate,
    HolonomicQubitRepeaterAuditReport, HolonomicQubitRepeaterParams,
    HolonomicQubitRepeaterProcessor, ParityReadoutSpectrumPoint, RepeaterNodeMetricPoint,
    ValleySpectrumPoint, ValleyWaveguidePoint,
};
use std::time::Instant;

/// 5 Categorized navigation tabs for the Holonomic Qubit Repeater dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HolonomicQubitRepeaterTab {
    HolonomicBraiding,
    FractionalValleyRouter,
    CryogenicCoprocessor,
    CornerLatticeCanvas,
    AuditTelemetry,
}

impl HolonomicQubitRepeaterTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::HolonomicBraiding => "Holonomic Braiding",
            Self::FractionalValleyRouter => "Fractional Valley Router",
            Self::CryogenicCoprocessor => "Cryogenic Co-Processor",
            Self::CornerLatticeCanvas => "2D Corner Lattice Canvas",
            Self::AuditTelemetry => "Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Phase 468.
#[derive(Debug, Clone)]
pub struct HolonomicQubitRepeaterDialog {
    pub is_open: bool,
    pub active_tab: HolonomicQubitRepeaterTab,

    // Holonomic braiding parameters
    pub center_freq_ghz: f64,
    pub intracell_gamma_mhz: f64,
    pub intercell_lambda_mhz: f64,
    pub braid_duration_ns: f64,
    pub selected_gate: HolonomicQubitGate,

    // Fractional valley router parameters
    pub valley_staggering_mhz: f64,
    pub synthetic_gauge_phase_rad: f64,
    pub waveguide_length_um: f64,
    pub corner_bend_angle_deg: f64,
    pub has_obstacle_defect: bool,

    // Cryogenic co-processor parameters
    pub operating_temperature_mk: f64,
    pub dispersive_coupling_chi_mhz: f64,
    pub cavity_decay_kappa_mhz: f64,
    pub clock_rate_mhz: f64,
    pub num_repeater_nodes: usize,

    // Cached physics telemetry
    pub cached_braiding_metrics: HolonomicBraidingMetrics,
    pub cached_spatial_points: Vec<HolonomicCornerSpatialPoint>,
    pub cached_loop_points: Vec<HolonomicLoopPoint>,
    pub cached_valley_metrics: FractionalValleyMetrics,
    pub cached_valley_spectrum: Vec<ValleySpectrumPoint>,
    pub cached_waveguide_points: Vec<ValleyWaveguidePoint>,
    pub cached_cryo_metrics: HolonomicCryoMetrics,
    pub cached_parity_spectrum: Vec<ParityReadoutSpectrumPoint>,
    pub cached_repeater_nodes: Vec<RepeaterNodeMetricPoint>,
    pub cached_audit: HolonomicQubitRepeaterAuditReport,
    pub last_solve_time_us: u64,
}

impl Default for HolonomicQubitRepeaterDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl HolonomicQubitRepeaterDialog {
    /// Sub-2.0 ms cold-boot constructor with pre-seeded baseline telemetry.
    pub fn new_fast() -> Self {
        let center_freq_ghz = 4.20;
        let intracell_gamma_mhz = 2.50;
        let intercell_lambda_mhz = 12.0;
        let braid_duration_ns = 90.0;
        let selected_gate = HolonomicQubitGate::Hadamard;

        let valley_staggering_mhz = 18.0;
        let synthetic_gauge_phase_rad = 2.0 * std::f64::consts::PI / 3.0;
        let waveguide_length_um = 120.0;
        let corner_bend_angle_deg = 60.0;
        let has_obstacle_defect = false;

        let operating_temperature_mk = 15.0;
        let dispersive_coupling_chi_mhz = 4.50;
        let cavity_decay_kappa_mhz = 0.40;
        let clock_rate_mhz = 1.20;
        let num_repeater_nodes = 4;

        let params = HolonomicQubitRepeaterParams {
            braiding_params: HolonomicBraidingParams {
                bare_freq_ghz: center_freq_ghz,
                intracell_gamma_mhz,
                intercell_lambda_mhz,
                braid_duration_ns,
                target_gate: selected_gate,
                cavity_linewidth_mhz: 0.20,
                grid_size: 6,
            },
            valley_params: FractionalValleyParams {
                valley_staggering_mhz,
                synthetic_gauge_phase_rad,
                waveguide_length_um,
                corner_bend_angle_deg,
                has_obstacle_defect,
                acoustic_speed_ms: 3450.0,
            },
            cryo_params: HolonomicCryoParams {
                operating_temperature_mk,
                carrier_frequency_ghz: center_freq_ghz,
                dispersive_coupling_chi_mhz,
                cavity_decay_kappa_mhz,
                readout_integration_time_ns: 140.0,
                clock_rate_mhz,
                num_repeater_nodes,
            },
        };

        let start = Instant::now();
        let solution = HolonomicQubitRepeaterProcessor::solve(&params);
        let audit = HolonomicQubitRepeaterProcessor::audit_coprocessor(&params);
        let last_solve_time_us = start.elapsed().as_micros() as u64;

        Self {
            is_open: false,
            active_tab: HolonomicQubitRepeaterTab::HolonomicBraiding,
            center_freq_ghz,
            intracell_gamma_mhz,
            intercell_lambda_mhz,
            braid_duration_ns,
            selected_gate,
            valley_staggering_mhz,
            synthetic_gauge_phase_rad,
            waveguide_length_um,
            corner_bend_angle_deg,
            has_obstacle_defect,
            operating_temperature_mk,
            dispersive_coupling_chi_mhz,
            cavity_decay_kappa_mhz,
            clock_rate_mhz,
            num_repeater_nodes,
            cached_braiding_metrics: solution.braiding_metrics,
            cached_spatial_points: solution.spatial_points,
            cached_loop_points: solution.loop_points,
            cached_valley_metrics: solution.valley_metrics,
            cached_valley_spectrum: solution.valley_spectrum,
            cached_waveguide_points: solution.waveguide_points,
            cached_cryo_metrics: solution.cryo_metrics,
            cached_parity_spectrum: solution.parity_spectrum,
            cached_repeater_nodes: solution.repeater_nodes,
            cached_audit: audit,
            last_solve_time_us,
        }
    }

    /// Recomputes simulation telemetry across all sub-engines.
    pub fn recompute(&mut self) {
        let params = HolonomicQubitRepeaterParams {
            braiding_params: HolonomicBraidingParams {
                bare_freq_ghz: self.center_freq_ghz,
                intracell_gamma_mhz: self.intracell_gamma_mhz,
                intercell_lambda_mhz: self.intercell_lambda_mhz,
                braid_duration_ns: self.braid_duration_ns,
                target_gate: self.selected_gate,
                cavity_linewidth_mhz: 0.20,
                grid_size: 6,
            },
            valley_params: FractionalValleyParams {
                valley_staggering_mhz: self.valley_staggering_mhz,
                synthetic_gauge_phase_rad: self.synthetic_gauge_phase_rad,
                waveguide_length_um: self.waveguide_length_um,
                corner_bend_angle_deg: self.corner_bend_angle_deg,
                has_obstacle_defect: self.has_obstacle_defect,
                acoustic_speed_ms: 3450.0,
            },
            cryo_params: HolonomicCryoParams {
                operating_temperature_mk: self.operating_temperature_mk,
                carrier_frequency_ghz: self.center_freq_ghz,
                dispersive_coupling_chi_mhz: self.dispersive_coupling_chi_mhz,
                cavity_decay_kappa_mhz: self.cavity_decay_kappa_mhz,
                readout_integration_time_ns: 140.0,
                clock_rate_mhz: self.clock_rate_mhz,
                num_repeater_nodes: self.num_repeater_nodes,
            },
        };

        let start = Instant::now();
        let solution = HolonomicQubitRepeaterProcessor::solve(&params);
        let audit = HolonomicQubitRepeaterProcessor::audit_coprocessor(&params);
        self.last_solve_time_us = start.elapsed().as_micros() as u64;

        self.cached_braiding_metrics = solution.braiding_metrics;
        self.cached_spatial_points = solution.spatial_points;
        self.cached_loop_points = solution.loop_points;
        self.cached_valley_metrics = solution.valley_metrics;
        self.cached_valley_spectrum = solution.valley_spectrum;
        self.cached_waveguide_points = solution.waveguide_points;
        self.cached_cryo_metrics = solution.cryo_metrics;
        self.cached_parity_spectrum = solution.parity_spectrum;
        self.cached_repeater_nodes = solution.repeater_nodes;
        self.cached_audit = audit;
    }

    /// Renders the modal window UI.
    pub fn ui(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut open = self.is_open;
        Window::new("Non-Abelian Holonomic Qubit Braiding & Fractional Valley Repeater (Phase 468)")
            .open(&mut open)
            .default_size(Vec2::new(940.0, 680.0))
            .min_size(Vec2::new(820.0, 560.0))
            .resizable(true)
            .show(ctx, |ui| {
                self.render_header(ui);
                ui.separator();

                self.render_tab_bar(ui);
                ui.separator();

                self.render_content(ui);

                ui.separator();
                self.render_footer(ui);
            });
        self.is_open = open;
    }

    /// Renders the active tab content within the dialog.
    pub fn render_content(&mut self, ui: &mut Ui) {
        match self.active_tab {
            HolonomicQubitRepeaterTab::HolonomicBraiding => self.render_braiding_tab(ui),
            HolonomicQubitRepeaterTab::FractionalValleyRouter => self.render_valley_tab(ui),
            HolonomicQubitRepeaterTab::CryogenicCoprocessor => self.render_cryo_tab(ui),
            HolonomicQubitRepeaterTab::CornerLatticeCanvas => self.render_canvas_tab(ui),
            HolonomicQubitRepeaterTab::AuditTelemetry => self.render_audit_tab(ui),
        }
    }

    fn render_header(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Topological Acoustic Non-Abelian Holonomic Qubit Co-Processor")
                    .strong()
                    .size(15.0)
                    .color(Color32::from_rgb(180, 110, 255)),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (status_text, status_color) = if self.cached_audit.is_all_pass() {
                    ("10/10 PASS - HOLONOMIC CO-PROCESSOR CERTIFIED", Color32::from_rgb(60, 220, 120))
                } else {
                    ("AUDIT DEGRADED", Color32::from_rgb(250, 70, 70))
                };
                ui.label(RichText::new(status_text).strong().size(12.0).color(status_color));
            });
        });
    }

    fn render_tab_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let tabs = [
                HolonomicQubitRepeaterTab::HolonomicBraiding,
                HolonomicQubitRepeaterTab::FractionalValleyRouter,
                HolonomicQubitRepeaterTab::CryogenicCoprocessor,
                HolonomicQubitRepeaterTab::CornerLatticeCanvas,
                HolonomicQubitRepeaterTab::AuditTelemetry,
            ];

            for tab in tabs {
                let is_selected = self.active_tab == tab;
                let text = if is_selected {
                    RichText::new(tab.label()).strong().color(Color32::from_rgb(240, 210, 255))
                } else {
                    RichText::new(tab.label()).color(Color32::from_rgb(160, 175, 195))
                };

                if ui.selectable_label(is_selected, text).clicked() {
                    self.active_tab = tab;
                }
            }
        });
    }

    fn render_braiding_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(RichText::new("Higher-Order Corner Manifold & Wilczek-Zee Holonomic Gates").strong());
            ui.add_space(6.0);

            let mut changed = false;
            ui.horizontal(|ui| {
                ui.label("Center Frequency (GHz):");
                changed |= ui.add(egui::Slider::new(&mut self.center_freq_ghz, 2.0..=8.0).text("GHz")).changed();

                ui.label("Intracell Gamma (MHz):");
                changed |= ui.add(egui::Slider::new(&mut self.intracell_gamma_mhz, 0.5..=6.0).text("MHz")).changed();
            });

            ui.horizontal(|ui| {
                ui.label("Intercell Lambda (MHz):");
                changed |= ui.add(egui::Slider::new(&mut self.intercell_lambda_mhz, 6.0..=20.0).text("MHz")).changed();

                ui.label("Braid Duration (ns):");
                changed |= ui.add(egui::Slider::new(&mut self.braid_duration_ns, 40.0..=200.0).text("ns")).changed();
            });

            ui.horizontal(|ui| {
                ui.label("Target Holonomic Gate:");
                let current_gate = self.selected_gate;
                egui::ComboBox::from_id_salt("holonomic_gate_select")
                    .selected_text(current_gate.name())
                    .show_ui(ui, |ui| {
                        let gates = [
                            HolonomicQubitGate::Hadamard,
                            HolonomicQubitGate::PhaseS,
                            HolonomicQubitGate::PauliX,
                            HolonomicQubitGate::PauliZ,
                            HolonomicQubitGate::TGate,
                            HolonomicQubitGate::ControlledPhase,
                        ];
                        for g in gates {
                            if ui.selectable_label(self.selected_gate == g, g.name()).clicked() {
                                self.selected_gate = g;
                                changed = true;
                            }
                        }
                    });
            });

            if changed {
                self.recompute();
            }

            ui.add_space(8.0);
            ui.separator();

            let m = &self.cached_braiding_metrics;
            ui.columns(3, |cols| {
                cols[0].label(RichText::new("Bulk Bandgap:").strong());
                cols[0].label(format!("{:.2} MHz", m.bulk_bandgap_mhz));
                cols[0].label(RichText::new("Quadrupole Moment q_xy:").strong());
                cols[0].label(format!("{:.2}", m.quadrupole_moment));

                cols[1].label(RichText::new("Corner Confinement:").strong());
                cols[1].label(format!("{:.1}%", m.corner_confinement_ratio * 100.0));
                cols[1].label(RichText::new("Commutator ||[H, S]||:").strong());
                cols[1].label(format!("{:.3}", m.non_abelian_commutator_norm));

                cols[2].label(RichText::new("Gate Process Fidelity:").strong());
                cols[2].label(format!("{:.3}%", m.gate_process_fidelity * 100.0));
                cols[2].label(RichText::new("Diabatic Leakage:").strong());
                cols[2].label(format!("{:.2e}", m.diabatic_leakage_rate));
            });

            ui.add_space(10.0);
            ui.label(RichText::new("Wilczek-Zee Holonomic Control Loop Trajectory:").strong());
            ui.label(format!(
                "Synthesized Gate: {} | Accumulated Geometric Phase: {:.3} rad",
                self.selected_gate.name(),
                m.geometric_phase_rad
            ));
        });
    }

    fn render_valley_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(RichText::new("Valley-Chern Waveguide Routing & Fractional Charge Repeater").strong());
            ui.add_space(6.0);

            let mut changed = false;
            ui.horizontal(|ui| {
                ui.label("Valley Staggering (MHz):");
                changed |= ui.add(egui::Slider::new(&mut self.valley_staggering_mhz, 5.0..=35.0).text("MHz")).changed();

                ui.label("Waveguide Length (um):");
                changed |= ui.add(egui::Slider::new(&mut self.waveguide_length_um, 50.0..=300.0).text("um")).changed();
            });

            ui.horizontal(|ui| {
                ui.label("Bend Angle (deg):");
                changed |= ui.add(egui::Slider::new(&mut self.corner_bend_angle_deg, 30.0..=120.0).text("deg")).changed();

                changed |= ui.checkbox(&mut self.has_obstacle_defect, "Inject Vacancy Defect at Corner").changed();
            });

            if changed {
                self.recompute();
            }

            ui.add_space(8.0);
            ui.separator();

            let m = &self.cached_valley_metrics;
            ui.columns(3, |cols| {
                cols[0].label(RichText::new("Valley Chern Contrast Delta Cv:").strong());
                cols[0].label(format!("{}", m.valley_chern_contrast));
                cols[0].label(RichText::new("Fractional Valley Charge Q_frac:").strong());
                cols[0].label(format!("{:.3} e (e/3)", m.fractional_valley_charge));

                cols[1].label(RichText::new("Forward Insertion Loss:").strong());
                cols[1].label(format!("{:.2} dB", m.forward_insertion_loss_db));
                cols[1].label(RichText::new("Reverse Chiral Isolation:").strong());
                cols[1].label(format!("{:.1} dB", m.reverse_chiral_isolation_db));

                cols[2].label(RichText::new("Waveguide Directivity:").strong());
                cols[2].label(format!("{:.1} dB", m.directivity_db));
                cols[2].label(RichText::new("Bend Defect Retention:").strong());
                cols[2].label(format!("{:.1}%", m.defect_immunity_ratio * 100.0));
            });

            ui.add_space(8.0);
            ui.label(RichText::new("Valley Waveguide Transmission Spectrum [4.0 - 4.4 GHz]:").strong());
            ui.label(format!(
                "Chiral Group Velocity: {:.1} m/s | S21 Peak: -{:.2} dB | S12 Isolation: -{:.1} dB",
                m.chiral_group_velocity_ms, m.forward_insertion_loss_db, m.reverse_chiral_isolation_db
            ));
        });
    }

    fn render_cryo_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(RichText::new("15 mK Dilution Refrigerator Operation & CV Entanglement Repeater").strong());
            ui.add_space(6.0);

            let mut changed = false;
            ui.horizontal(|ui| {
                ui.label("Operating Temperature (mK):");
                changed |= ui.add(egui::Slider::new(&mut self.operating_temperature_mk, 5.0..=50.0).text("mK")).changed();

                ui.label("Dispersive Coupling Chi (MHz):");
                changed |= ui.add(egui::Slider::new(&mut self.dispersive_coupling_chi_mhz, 1.0..=10.0).text("MHz")).changed();
            });

            ui.horizontal(|ui| {
                ui.label("Cavity Linewidth Kappa (MHz):");
                changed |= ui.add(egui::Slider::new(&mut self.cavity_decay_kappa_mhz, 0.1..=1.5).text("MHz")).changed();

                ui.label("Clock Rate (MHz):");
                changed |= ui.add(egui::Slider::new(&mut self.clock_rate_mhz, 0.5..=3.0).text("MHz")).changed();
            });

            ui.horizontal(|ui| {
                ui.label("Repeater Node Count:");
                changed |= ui.add(egui::Slider::new(&mut self.num_repeater_nodes, 2..=8)).changed();
            });

            if changed {
                self.recompute();
            }

            ui.add_space(8.0);
            ui.separator();

            let m = &self.cached_cryo_metrics;
            ui.columns(3, |cols| {
                cols[0].label(RichText::new("Thermal Phonon Occupancy:").strong());
                cols[0].label(format!("{:.2e} quanta", m.thermal_phonon_occupancy));
                cols[0].label(RichText::new("Quadrature Squeezing:").strong());
                cols[0].label(format!("{:.2} dB below SQL", m.quadrature_squeezing_db));

                cols[1].label(RichText::new("Duan-Simon EPR Nullifier:").strong());
                cols[1].label(format!("{:.3} (inseparable < 1.0)", m.duan_simon_nullifier));
                cols[1].label(RichText::new("Entanglement Swap Fidelity:").strong());
                cols[1].label(format!("{:.3}%", m.entanglement_swap_fidelity * 100.0));

                cols[2].label(RichText::new("Parity Doublet Splitting:").strong());
                cols[2].label(format!("{:.1} MHz", m.dispersive_doublet_splitting_mhz));
                cols[2].label(RichText::new("Readout SNR / Fidelity:").strong());
                cols[2].label(format!("{:.1} dB / {:.3}%", m.readout_snr_db, m.single_shot_readout_fidelity * 100.0));
            });

            ui.add_space(8.0);
            ui.label(format!("Cryo-CMOS Power Dissipation: {:.2} mW (dilution fridge thermal budget <= 0.8 mW)", m.cryo_power_dissipation_mw));
        });
    }

    fn render_canvas_tab(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(RichText::new("2D Higher-Order Quadrupole Lattice & Valley Router Canvas").strong());
            ui.label("Visualizes 4-corner bound states, chiral domain boundary, and distributed repeater nodes.");
            ui.add_space(6.0);

            let (response, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), 320.0), Sense::hover());
            let rect = response.rect;

            painter.rect_filled(rect, 4.0, Color32::from_rgb(18, 22, 30));
            painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(50, 60, 80)), StrokeKind::Outside);

            let pad = 24.0;
            let canvas_w = rect.width() - 2.0 * pad;
            let canvas_h = rect.height() - 2.0 * pad;

            // Render 2D lattice spatial points
            for pt in &self.cached_spatial_points {
                let px = rect.min.x + pad + pt.x as f32 * canvas_w;
                let py = rect.min.y + pad + pt.y as f32 * canvas_h;

                let (color, radius) = if pt.is_corner {
                    (Color32::from_rgb(255, 90, 220), 8.0)
                } else {
                    let intensity_byte = (pt.intensity * 200.0) as u8;
                    (Color32::from_rgb(40 + intensity_byte / 3, 70 + intensity_byte / 2, 130 + intensity_byte / 2), 3.5)
                };

                painter.circle_filled(egui::pos2(px, py), radius, color);
            }

            // Draw domain wall routing line
            let p1 = egui::pos2(rect.min.x + pad, rect.min.y + pad + canvas_h * 0.5);
            let p_bend = egui::pos2(rect.min.x + pad + canvas_w * 0.5, rect.min.y + pad + canvas_h * 0.5);
            let p2 = egui::pos2(rect.min.x + pad + canvas_w * 0.85, rect.min.y + pad + canvas_h * 0.2);

            painter.line_segment([p1, p_bend], Stroke::new(2.5, Color32::from_rgb(80, 220, 140)));
            painter.line_segment([p_bend, p2], Stroke::new(2.5, Color32::from_rgb(80, 220, 140)));

            // Legend labels
            painter.text(
                egui::pos2(rect.min.x + 12.0, rect.min.y + 12.0),
                egui::Align2::LEFT_TOP,
                "Magenta Nodes: Corner Bound Qubits (C1..C4) | Green Wave: Chiral Valley Bus",
                egui::FontId::proportional(11.0),
                Color32::from_rgb(200, 215, 235),
            );
        });
    }

    fn render_audit_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(RichText::new("10-Point Rigorous Physics Invariant Audit Checklist").strong().size(14.0));
            ui.add_space(4.0);

            let audit = &self.cached_audit;
            let checks = [
                ("1. Higher-Order Corner Energy Confinement", audit.corner_energy_confinement_pass, "eta_corner >= 90.0% zero-mode energy density"),
                ("2. Non-Abelian Non-Commutativity", audit.non_abelian_commutator_pass, "||[U_H, U_S]|| >= 0.70 Frobenius commutator norm"),
                ("3. Wilczek-Zee Holonomic Gate Fidelity", audit.holonomic_gate_fidelity_pass, "F_gate >= 99.8% across target gates"),
                ("4. Adiabatic Leakage Suppression", audit.adiabatic_leakage_suppression_pass, "P_leak <= 1.0e-4 Landau-Zener transition rate"),
                ("5. Valley Chern Contrast Quantization", audit.valley_chern_contrast_pass, "Delta Cv = 2 quantized topological contrast"),
                ("6. Fractional Valley Charge Accumulation", audit.fractional_valley_charge_pass, "Q_frac = e/3 (0.333) third-fractional charge"),
                ("7. Reverse Chiral Valley Isolation", audit.reverse_chiral_isolation_pass, "ISO >= 42.0 dB non-reciprocal isolation"),
                ("8. 15 mK Thermal Phonon Occupancy", audit.thermal_phonon_occupancy_pass, "n_th <= 1.0e-4 dilution refrigerator ground state"),
                ("9. CV Squeezing & Duan-Simon EPR Nullifier", audit.duan_simon_nullifier_pass, "Delta_EPR <= 0.35 < 1.0 inseparability certified"),
                ("10. Dispersive Parity Readout SNR & Fidelity", audit.dispersive_readout_snr_pass, "SNR >= 17.0 dB, single-shot fidelity >= 99.8%"),
            ];

            for (title, passed, detail) in checks {
                ui.horizontal(|ui| {
                    let (badge_text, badge_color) = if passed {
                        ("[PASS]", Color32::from_rgb(60, 220, 100))
                    } else {
                        ("[FAIL]", Color32::from_rgb(250, 70, 70))
                    };

                    ui.label(RichText::new(badge_text).strong().monospace().color(badge_color));
                    ui.label(RichText::new(title).strong());
                    ui.label(RichText::new(format!("- {detail}")).size(11.0).color(Color32::from_rgb(170, 185, 205)));
                });
            }

            ui.add_space(10.0);
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Re-run Physics Audit").clicked() {
                    self.recompute();
                }

                if ui.button("Reset to Calibrated Baseline").clicked() {
                    *self = Self::new_fast();
                    self.is_open = true;
                }
            });
        });
    }

    fn render_footer(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "Audit Score: {}/{} PASS | Solve Latency: {} us",
                    self.cached_audit.passed_count,
                    self.cached_audit.total_count,
                    self.last_solve_time_us
                ))
                .size(11.0)
                .color(Color32::from_rgb(150, 165, 185)),
            );
        });
    }
}
