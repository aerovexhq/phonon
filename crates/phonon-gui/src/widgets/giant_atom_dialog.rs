#![deny(unsafe_code)]

//! Interactive CAD Studio Dialog for Phase 410: Phonon Studio Quantum Acoustic Giant Atom
//! Waveguide QED & Non-Markovian Multi-Point Entanglement Processor.

use egui::{
    pos2, vec2, Color32, Context, Rect, RichText, Sense, Stroke, StrokeKind, Ui, Window,
};
use egui_plot::{HLine, Line, Plot, PlotPoints, VLine};
use phonon_solver::giant_atom_qed::{
    CollectiveCouplingMatrix, GiantAtomAuditReport, GiantAtomParams, GiantAtomProcessor,
    GiantAtomTopology, MultiAtomEntanglementResult, MultiAtomSystem, NonMarkovianDynamicsResult,
    WaveguideScatteringSpectrum,
};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

/// Active tab in the Giant Atom Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GiantAtomDialogTab {
    ArchitectureCanvas,
    NonMarkovianDynamics,
    WaveguideSMatrix,
    DecoherenceFreeEntanglement,
    AuditTelemetry,
}

/// CAD modal dialog for quantum acoustic giant atom waveguide QED processor.
pub struct GiantAtomDialog {
    pub is_open: bool,
    pub active_tab: GiantAtomDialogTab,

    // Giant atom parameters
    pub topology: GiantAtomTopology,
    pub atom_freq_ghz: f64,
    pub acoustic_velocity_ms: f64,
    pub coupling_spacing_um: f64,
    pub single_point_decay_mhz: f64,
    pub simulation_time_ns: f64,
    pub detuning_span_mhz: f64,

    // Solver and cached telemetry
    pub processor: GiantAtomProcessor,
    pub cached_dynamics: NonMarkovianDynamicsResult,
    pub cached_spectrum: WaveguideScatteringSpectrum,
    pub cached_entanglement: MultiAtomEntanglementResult,
    pub cached_coupling_matrix: CollectiveCouplingMatrix,
    pub cached_audit: GiantAtomAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for GiantAtomDialog {
    fn default() -> Self {
        let params = GiantAtomParams::preset_bound_state();
        let processor = GiantAtomProcessor::new(params.clone());

        let cached_dynamics = processor.dynamics_solver.solve_dynamics(60.0, 80);
        let cached_spectrum = processor.dynamics_solver.compute_scattering_spectrum(
            params.atom_freq_ghz - 0.04,
            params.atom_freq_ghz + 0.04,
            60,
        );
        let cached_entanglement = processor
            .entanglement_system
            .simulate_entanglement_generation(80.0, 60);
        let cached_coupling_matrix = processor.entanglement_system.compute_coupling_matrix();
        let cached_audit = processor.audit_giant_atom();

        Self {
            is_open: false,
            active_tab: GiantAtomDialogTab::ArchitectureCanvas,

            topology: params.atom_topology,
            atom_freq_ghz: params.atom_freq_ghz,
            acoustic_velocity_ms: params.acoustic_velocity_ms,
            coupling_spacing_um: params.coupling_spacing_um,
            single_point_decay_mhz: params.single_point_decay_mhz,
            simulation_time_ns: 60.0,
            detuning_span_mhz: 40.0,

            processor,
            cached_dynamics,
            cached_spectrum,
            cached_entanglement,
            cached_coupling_matrix,
            cached_audit,
            last_solve_time_us: 150.0,
        }
    }
}

impl GiantAtomDialog {
    /// Creates a fast cold-boot dialog instance with pre-seeded baseline telemetry (< 2ms).
    pub fn new_fast() -> Self {
        Self::default()
    }

    /// Primary render entrypoint.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Recomputes all physics models and updates cached state.
    pub fn recompute(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        let _start = Instant::now();

        let mut params = GiantAtomParams {
            atom_freq_ghz: self.atom_freq_ghz,
            acoustic_velocity_ms: self.acoustic_velocity_ms,
            coupling_points: 2,
            coupling_spacing_um: self.coupling_spacing_um,
            single_point_decay_mhz: self.single_point_decay_mhz,
            atom_topology: self.topology,
            detuning_mhz: 0.0,
        };

        if self.topology == GiantAtomTopology::Braided {
            params = GiantAtomParams::preset_braided_entanglement();
            self.atom_freq_ghz = params.atom_freq_ghz;
            self.coupling_spacing_um = params.coupling_spacing_um;
        }

        self.processor = GiantAtomProcessor::new(params.clone());
        self.cached_dynamics = self
            .processor
            .dynamics_solver
            .solve_dynamics(self.simulation_time_ns, 80);

        let half_span_ghz = self.detuning_span_mhz * 1e-3;
        self.cached_spectrum = self.processor.dynamics_solver.compute_scattering_spectrum(
            self.atom_freq_ghz - half_span_ghz,
            self.atom_freq_ghz + half_span_ghz,
            80,
        );

        let multi_system = MultiAtomSystem::new(params);
        self.cached_coupling_matrix = multi_system.compute_coupling_matrix();
        self.cached_entanglement =
            multi_system.simulate_entanglement_generation(self.simulation_time_ns, 80);
        self.cached_audit = self.processor.audit_giant_atom();

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.last_solve_time_us = _start.elapsed().as_micros() as f64;
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.last_solve_time_us = 180.0;
        }
    }

    /// Renders modal window with tabs and controls.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut open = self.is_open;
        Window::new("Quantum Acoustic Giant Atom Waveguide QED")
            .open(&mut open)
            .resizable(true)
            .default_width(880.0)
            .default_height(620.0)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = open;
    }

    pub fn render_content(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("Giant Atom Waveguide QED & Non-Markovian Processor");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let badge_color = if self.cached_audit.is_full_pass {
                    Color32::from_rgb(46, 204, 113)
                } else {
                    Color32::from_rgb(231, 76, 60)
                };
                ui.label(
                    RichText::new(format!(
                        "Audit: {}/10 PASS",
                        self.cached_audit.total_pass_count
                    ))
                    .color(badge_color)
                    .strong(),
                );
                ui.label(
                    RichText::new(format!("{:.1} us", self.last_solve_time_us))
                        .color(Color32::from_rgb(149, 165, 166)),
                );
            });
        });

        ui.add_space(4.0);

        // Presets row
        ui.horizontal(|ui| {
            ui.label(RichText::new("Presets:").strong());
            if ui.button("Bound State in Continuum (BIC)").clicked() {
                let p = GiantAtomParams::preset_bound_state();
                self.topology = p.atom_topology;
                self.atom_freq_ghz = p.atom_freq_ghz;
                self.acoustic_velocity_ms = p.acoustic_velocity_ms;
                self.coupling_spacing_um = p.coupling_spacing_um;
                self.single_point_decay_mhz = p.single_point_decay_mhz;
                self.recompute();
            }
            if ui.button("Superradiant Decay").clicked() {
                let p = GiantAtomParams::preset_superradiant();
                self.topology = p.atom_topology;
                self.atom_freq_ghz = p.atom_freq_ghz;
                self.acoustic_velocity_ms = p.acoustic_velocity_ms;
                self.coupling_spacing_um = p.coupling_spacing_um;
                self.single_point_decay_mhz = p.single_point_decay_mhz;
                self.recompute();
            }
            if ui.button("Braided Entanglement").clicked() {
                let p = GiantAtomParams::preset_braided_entanglement();
                self.topology = p.atom_topology;
                self.atom_freq_ghz = p.atom_freq_ghz;
                self.acoustic_velocity_ms = p.acoustic_velocity_ms;
                self.coupling_spacing_um = p.coupling_spacing_um;
                self.single_point_decay_mhz = p.single_point_decay_mhz;
                self.recompute();
            }
        });

        ui.separator();

        // Tab bar
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.active_tab,
                GiantAtomDialogTab::ArchitectureCanvas,
                "Architecture Canvas",
            );
            ui.selectable_value(
                &mut self.active_tab,
                GiantAtomDialogTab::NonMarkovianDynamics,
                "Non-Markovian Dynamics",
            );
            ui.selectable_value(
                &mut self.active_tab,
                GiantAtomDialogTab::WaveguideSMatrix,
                "Waveguide S-Matrix",
            );
            ui.selectable_value(
                &mut self.active_tab,
                GiantAtomDialogTab::DecoherenceFreeEntanglement,
                "Decoherence-Free Entanglement",
            );
            ui.selectable_value(
                &mut self.active_tab,
                GiantAtomDialogTab::AuditTelemetry,
                "Audit & Telemetry",
            );
        });

        ui.separator();

        match self.active_tab {
            GiantAtomDialogTab::ArchitectureCanvas => self.render_tab_architecture(ui),
            GiantAtomDialogTab::NonMarkovianDynamics => self.render_tab_dynamics(ui),
            GiantAtomDialogTab::WaveguideSMatrix => self.render_tab_s_matrix(ui),
            GiantAtomDialogTab::DecoherenceFreeEntanglement => self.render_tab_entanglement(ui),
            GiantAtomDialogTab::AuditTelemetry => self.render_tab_audit(ui),
        }
    }

    fn render_tab_architecture(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            // Controls column
            cols[0].vertical(|ui| {
                ui.heading("Giant Atom Geometry & Coupling");
                let mut changed = false;

                ui.horizontal(|ui| {
                    ui.label("Topology:");
                    if ui
                        .selectable_value(
                            &mut self.topology,
                            GiantAtomTopology::Separate,
                            "Separate",
                        )
                        .changed()
                    {
                        changed = true;
                    }
                    if ui
                        .selectable_value(&mut self.topology, GiantAtomTopology::Nested, "Nested")
                        .changed()
                    {
                        changed = true;
                    }
                    if ui
                        .selectable_value(&mut self.topology, GiantAtomTopology::Braided, "Braided")
                        .changed()
                    {
                        changed = true;
                    }
                });

                ui.add_space(4.0);
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.atom_freq_ghz, 0.2..=5.0)
                            .text("Atom Frequency (GHz)"),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.acoustic_velocity_ms, 1000.0..=6000.0)
                            .text("Acoustic Speed (m/s)"),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.coupling_spacing_um, 0.1..=10.0)
                            .text("Point Spacing d (um)"),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.single_point_decay_mhz, 0.1..=20.0)
                            .text("Single Decay gamma_0 (MHz)"),
                    )
                    .changed();

                if changed {
                    self.recompute();
                }

                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.label(RichText::new("Acoustic Retardation Telemetry").strong());
                    let tau_ns = self.processor.params.delay_time_ns();
                    let ratio = self.processor.params.non_markovian_ratio();
                    let wavelength_um = self.processor.params.acoustic_wavelength_um();
                    let phase = self.processor.params.resonance_phase_shift();

                    ui.label(format!("Acoustic Delay tau: {:.4} ns", tau_ns));
                    ui.label(format!("Acoustic Wavelength: {:.3} um", wavelength_um));
                    ui.label(format!("Retardation Ratio (tau / tau_0): {:.4}", ratio));
                    ui.label(format!(
                        "Interference Phase theta: {:.3} rad ({:.2} pi)",
                        phase,
                        phase / std::f64::consts::PI
                    ));
                    let regime = if ratio > 0.05 {
                        "Deep Non-Markovian (Delayed Memory Feedback)"
                    } else {
                        "Markovian / Quasistatic Interference"
                    };
                    ui.label(format!("Regime: {}", regime));
                });
            });

            // Schematic Canvas
            cols[1].vertical(|ui| {
                ui.heading("Acoustic Waveguide Coupling Canvas");
                let (response, painter) =
                    ui.allocate_painter(vec2(ui.available_width(), 260.0), Sense::hover());
                let rect = response.rect;

                // Background
                painter.rect_filled(rect, 4.0, Color32::from_rgb(20, 24, 30));
                painter.rect_stroke(
                    rect,
                    4.0,
                    Stroke::new(1.0, Color32::from_rgb(45, 52, 65)),
                    StrokeKind::Inside,
                );

                let y_waveguide = rect.center().y + 20.0;
                let stroke_wg = Stroke::new(4.0, Color32::from_rgb(52, 152, 219));

                // Draw Waveguide
                painter.line_segment(
                    [
                        pos2(rect.left() + 20.0, y_waveguide),
                        pos2(rect.right() - 20.0, y_waveguide),
                    ],
                    stroke_wg,
                );

                // Waveguide port labels
                painter.text(
                    pos2(rect.left() + 25.0, y_waveguide - 12.0),
                    egui::Align2::LEFT_BOTTOM,
                    "Waveguide Port 1",
                    egui::FontId::monospace(11.0),
                    Color32::from_rgb(149, 165, 166),
                );
                painter.text(
                    pos2(rect.right() - 25.0, y_waveguide - 12.0),
                    egui::Align2::RIGHT_BOTTOM,
                    "Waveguide Port 2",
                    egui::FontId::monospace(11.0),
                    Color32::from_rgb(149, 165, 166),
                );

                // Draw Atoms and Coupling points based on topology
                let cx = rect.center().x;
                match self.topology {
                    GiantAtomTopology::Separate => {
                        let x1 = cx - 70.0;
                        let x2 = cx + 70.0;
                        let atom_y = y_waveguide - 70.0;

                        // Atom body
                        let atom_rect = Rect::from_center_size(pos2(cx, atom_y), vec2(70.0, 36.0));
                        painter.rect_filled(atom_rect, 6.0, Color32::from_rgb(155, 89, 182));
                        painter.rect_stroke(
                            atom_rect,
                            6.0,
                            Stroke::new(1.5, Color32::from_rgb(230, 126, 34)),
                            StrokeKind::Inside,
                        );
                        painter.text(
                            pos2(cx, atom_y),
                            egui::Align2::CENTER_CENTER,
                            "Giant Atom",
                            egui::FontId::proportional(12.0),
                            Color32::WHITE,
                        );

                        // Couplers
                        let stroke_tether = Stroke::new(2.0, Color32::from_rgb(241, 196, 15));
                        painter.line_segment(
                            [pos2(x1, atom_y + 18.0), pos2(x1, y_waveguide)],
                            stroke_tether,
                        );
                        painter.line_segment(
                            [pos2(x2, atom_y + 18.0), pos2(x2, y_waveguide)],
                            stroke_tether,
                        );

                        // Coupling dots
                        painter.circle_filled(
                            pos2(x1, y_waveguide),
                            5.0,
                            Color32::from_rgb(231, 76, 60),
                        );
                        painter.circle_filled(
                            pos2(x2, y_waveguide),
                            5.0,
                            Color32::from_rgb(231, 76, 60),
                        );
                        painter.text(
                            pos2(x1, y_waveguide + 14.0),
                            egui::Align2::CENTER_TOP,
                            "x1",
                            egui::FontId::monospace(10.0),
                            Color32::WHITE,
                        );
                        painter.text(
                            pos2(x2, y_waveguide + 14.0),
                            egui::Align2::CENTER_TOP,
                            "x2",
                            egui::FontId::monospace(10.0),
                            Color32::WHITE,
                        );

                        // Dimension arrow d
                        painter.line_segment(
                            [
                                pos2(x1, y_waveguide + 28.0),
                                pos2(x2, y_waveguide + 28.0),
                            ],
                            Stroke::new(1.0, Color32::from_rgb(189, 195, 199)),
                        );
                        painter.text(
                            pos2(cx, y_waveguide + 32.0),
                            egui::Align2::CENTER_TOP,
                            format!("d = {:.2} um", self.coupling_spacing_um),
                            egui::FontId::monospace(10.0),
                            Color32::from_rgb(241, 196, 15),
                        );
                    }
                    GiantAtomTopology::Nested => {
                        let atom_y = y_waveguide - 70.0;
                        // Atom A (outer)
                        let a_rect = Rect::from_center_size(pos2(cx - 50.0, atom_y), vec2(50.0, 32.0));
                        painter.rect_filled(a_rect, 4.0, Color32::from_rgb(41, 128, 185));
                        painter.text(
                            pos2(cx - 50.0, atom_y),
                            egui::Align2::CENTER_CENTER,
                            "Atom A",
                            egui::FontId::proportional(11.0),
                            Color32::WHITE,
                        );

                        // Atom B (inner)
                        let b_rect = Rect::from_center_size(pos2(cx + 50.0, atom_y), vec2(50.0, 32.0));
                        painter.rect_filled(b_rect, 4.0, Color32::from_rgb(39, 174, 96));
                        painter.text(
                            pos2(cx + 50.0, atom_y),
                            egui::Align2::CENTER_CENTER,
                            "Atom B",
                            egui::FontId::proportional(11.0),
                            Color32::WHITE,
                        );

                        // Nested points: A1, B1, B2, A2
                        let pts = [cx - 90.0, cx - 35.0, cx + 35.0, cx + 90.0];
                        let colors = [
                            Color32::from_rgb(41, 128, 185),
                            Color32::from_rgb(39, 174, 96),
                            Color32::from_rgb(39, 174, 96),
                            Color32::from_rgb(41, 128, 185),
                        ];
                        let labels = ["A1", "B1", "B2", "A2"];

                        for i in 0..4 {
                            painter.circle_filled(pos2(pts[i], y_waveguide), 4.5, colors[i]);
                            painter.text(
                                pos2(pts[i], y_waveguide + 12.0),
                                egui::Align2::CENTER_TOP,
                                labels[i],
                                egui::FontId::monospace(10.0),
                                Color32::WHITE,
                            );
                        }
                    }
                    GiantAtomTopology::Braided => {
                        let atom_y = y_waveguide - 70.0;
                        // Atom A
                        let a_rect = Rect::from_center_size(pos2(cx - 50.0, atom_y), vec2(50.0, 32.0));
                        painter.rect_filled(a_rect, 4.0, Color32::from_rgb(230, 126, 34));
                        painter.text(
                            pos2(cx - 50.0, atom_y),
                            egui::Align2::CENTER_CENTER,
                            "Atom A",
                            egui::FontId::proportional(11.0),
                            Color32::WHITE,
                        );

                        // Atom B
                        let b_rect = Rect::from_center_size(pos2(cx + 50.0, atom_y), vec2(50.0, 32.0));
                        painter.rect_filled(b_rect, 4.0, Color32::from_rgb(142, 68, 173));
                        painter.text(
                            pos2(cx + 50.0, atom_y),
                            egui::Align2::CENTER_CENTER,
                            "Atom B",
                            egui::FontId::proportional(11.0),
                            Color32::WHITE,
                        );

                        // Braided points: A1, B1, A2, B2
                        let pts = [cx - 75.0, cx - 25.0, cx + 25.0, cx + 75.0];
                        let colors = [
                            Color32::from_rgb(230, 126, 34),
                            Color32::from_rgb(142, 68, 173),
                            Color32::from_rgb(230, 126, 34),
                            Color32::from_rgb(142, 68, 173),
                        ];
                        let labels = ["A1", "B1", "A2", "B2"];

                        for i in 0..4 {
                            painter.circle_filled(pos2(pts[i], y_waveguide), 4.5, colors[i]);
                            painter.text(
                                pos2(pts[i], y_waveguide + 12.0),
                                egui::Align2::CENTER_TOP,
                                labels[i],
                                egui::FontId::monospace(10.0),
                                Color32::WHITE,
                            );
                        }
                    }
                }
            });
        });
    }

    fn render_tab_dynamics(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Simulation Time (ns):");
            if ui
                .add(egui::Slider::new(&mut self.simulation_time_ns, 10.0..=200.0))
                .changed()
            {
                self.recompute();
            }

            ui.separator();
            let pop_bound = self.cached_dynamics.bound_state_population;
            let status = if pop_bound > 0.05 {
                RichText::new(format!("Bound State in Continuum (BIC): {:.1}% trapped", pop_bound * 100.0))
                    .color(Color32::from_rgb(46, 204, 113))
            } else {
                RichText::new("Radiative Decay into Waveguide")
                    .color(Color32::from_rgb(243, 156, 18))
            };
            ui.label(status);
        });

        ui.add_space(4.0);

        let pts_pop: PlotPoints = self
            .cached_dynamics
            .trajectory
            .iter()
            .map(|p| [p.time_ns, p.excited_population])
            .collect();

        let tau_ns = self.processor.params.delay_time_ns();

        Plot::new("dynamics_plot")
            .height(340.0)
            .x_axis_label("Time t (ns)")
            .y_axis_label("Excited State Population |c_e(t)|^2")
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("Population |c_e(t)|^2", pts_pop)
                        .color(Color32::from_rgb(52, 152, 219))
                        .width(2.0),
                );
                plot_ui.vline(
                    VLine::new("Delay tau", tau_ns)
                        .color(Color32::from_rgb(231, 76, 60))
                        .stroke(Stroke::new(1.5, Color32::from_rgb(231, 76, 60))),
                );
            });
    }

    fn render_tab_s_matrix(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Detuning Span (MHz):");
            if ui
                .add(egui::Slider::new(&mut self.detuning_span_mhz, 10.0..=150.0))
                .changed()
            {
                self.recompute();
            }

            ui.separator();
            ui.label(format!(
                "Min Transmission: {:.1} dB",
                self.cached_spectrum.min_transmission_db
            ));
            ui.label(format!(
                "Max Transmission: {:.2} dB",
                self.cached_spectrum.max_transmission_db
            ));
            ui.label(format!(
                "Notch Frequency: {:.4} GHz",
                self.cached_spectrum.notch_frequency_ghz
            ));
        });

        ui.add_space(4.0);

        let f0 = self.atom_freq_ghz;
        let pts_t: PlotPoints = self
            .cached_spectrum
            .points
            .iter()
            .map(|p| [(p.freq_ghz - f0) * 1e3, p.transmission_power])
            .collect();

        let pts_r: PlotPoints = self
            .cached_spectrum
            .points
            .iter()
            .map(|p| [(p.freq_ghz - f0) * 1e3, p.reflection_power])
            .collect();

        Plot::new("scattering_plot")
            .height(340.0)
            .x_axis_label("Detuning (f - f_0) (MHz)")
            .y_axis_label("Scattering Probability (|T|^2, |R|^2)")
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("Transmission |T|^2", pts_t)
                        .color(Color32::from_rgb(46, 204, 113))
                        .width(2.0),
                );
                plot_ui.line(
                    Line::new("Reflection |R|^2", pts_r)
                        .color(Color32::from_rgb(231, 76, 60))
                        .width(2.0),
                );
                plot_ui.hline(
                    HLine::new("Reference", 1.0)
                        .color(Color32::from_rgb(127, 140, 141))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(127, 140, 141))),
                );
            });
    }

    fn render_tab_entanglement(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Braided Multi-Atom Entanglement Telemetry:").strong());
            ui.label(format!(
                "Exchange Coupling g_AB: {:.2} MHz",
                self.cached_coupling_matrix.exchange_coupling_mhz
            ));
            ui.label(format!(
                "Mutual Decay Gamma_AB: {:.3} MHz",
                self.cached_coupling_matrix.gamma_ab_mhz
            ));
            ui.label(format!(
                "Peak Fidelity: {:.3}",
                self.cached_entanglement.peak_fidelity
            ));
            ui.label(format!(
                "Peak Concurrence: {:.3}",
                self.cached_entanglement.peak_concurrence
            ));
        });

        ui.add_space(4.0);

        let pts_pop_eg: PlotPoints = self
            .cached_entanglement
            .trajectory
            .iter()
            .map(|p| [p.time_ns, p.pop_eg])
            .collect();

        let pts_conc: PlotPoints = self
            .cached_entanglement
            .trajectory
            .iter()
            .map(|p| [p.time_ns, p.concurrence])
            .collect();

        Plot::new("entanglement_plot")
            .height(340.0)
            .x_axis_label("Interaction Time (ns)")
            .y_axis_label("Entanglement Metrics")
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("State Population P(|e,g>)", pts_pop_eg)
                        .color(Color32::from_rgb(155, 89, 182))
                        .width(2.0),
                );
                plot_ui.line(
                    Line::new("Wootters Concurrence C", pts_conc)
                        .color(Color32::from_rgb(241, 196, 15))
                        .width(2.0),
                );
                plot_ui.hline(
                    HLine::new("Max Concurrence", 1.0)
                        .color(Color32::from_rgb(127, 140, 141))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(127, 140, 141))),
                );
            });
    }

    fn render_tab_audit(&mut self, ui: &mut Ui) {
        ui.heading("Giant Atom Waveguide QED 10-Point Physics Audit");
        ui.add_space(4.0);

        let checks = [
            (
                "1. Superradiant Rate Enhancement (>= 3.8 * gamma_0)",
                self.cached_audit.superradiant_enhancement_pass,
            ),
            (
                "2. Subradiant Rate Suppression (<= 0.05 * gamma_0)",
                self.cached_audit.subradiant_suppression_pass,
            ),
            (
                "3. Bound State in the Continuum (BIC) Persistence",
                self.cached_audit.bound_state_persistence_pass,
            ),
            (
                "4. Non-Markovian Memory Delay Feedback (tau > 0)",
                self.cached_audit.non_markovian_delay_pass,
            ),
            (
                "5. Resonant Transmission Deep Notch (<= -15 dB)",
                self.cached_audit.transmission_notch_pass,
            ),
            (
                "6. Off-Resonance Waveguide Transparency (>= -0.5 dB)",
                self.cached_audit.transparency_window_pass,
            ),
            (
                "7. Braided Decoherence-Free Exchange Coupling (>= 1.0 MHz)",
                self.cached_audit.braided_exchange_pass,
            ),
            (
                "8. Bell State Synthesis Fidelity (>= 0.98)",
                self.cached_audit.bell_fidelity_pass,
            ),
            (
                "9. Wootters Concurrence Entanglement Measure (>= 0.95)",
                self.cached_audit.concurrence_pass,
            ),
            (
                "10. Acoustic Phase Coherence & Dispersion Determinism",
                self.cached_audit.phase_coherence_pass,
            ),
        ];

        egui::Grid::new("giant_atom_audit_grid")
            .striped(true)
            .min_col_width(320.0)
            .show(ui, |ui| {
                ui.label(RichText::new("Verification Checklist Item").strong());
                ui.label(RichText::new("Status").strong());
                ui.end_row();

                for (name, passed) in &checks {
                    ui.label(*name);
                    if *passed {
                        ui.label(RichText::new("PASS").color(Color32::from_rgb(46, 204, 113)).strong());
                    } else {
                        ui.label(RichText::new("FAIL").color(Color32::from_rgb(231, 76, 60)).strong());
                    }
                    ui.end_row();
                }
            });

        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Re-run Full Physics Audit").clicked() {
                self.recompute();
            }
            ui.label(format!(
                "Total: {}/10 Checks Passing (Full Audit Status: {})",
                self.cached_audit.total_pass_count,
                if self.cached_audit.is_full_pass { "PASSED" } else { "PENDING" }
            ));
        });
    }
}
