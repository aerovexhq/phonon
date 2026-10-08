#![deny(unsafe_code)]

//! Interactive CAD Studio Dialog for Phase 425: Phonon Studio Topological Chiral
//! Acoustic Chern-Simons Fractional Anyon Interferometer & Non-Abelian Quantum Memory.
//!
//! Visualizes chiral acoustic Chern-Simons fractional anyon interferometry, Fabry-Perot
//! interference oscillations, statistical exchange phase jump extraction, topologically
//! protected anyonic quantum memory registers, real-space edge and QPC layout,
//! and a 10-point physics audit checklist.

use egui::{pos2, vec2, Color32, Context, Rect, RichText, Sense, Stroke, StrokeKind, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::chiral_chern_simons_interferometer::{
    AnyonicMemoryParams, ChernSimonsInterferometerParams, ChernSimonsMemoryAuditReport,
    ChernSimonsMemoryProcessor, FractionalAnyonKind, FractionalInterferenceMetrics,
    InterferometerTransmissionPoint, MemoryCoherenceDecayPoint,
    TopologicalMemoryStateReport,
};

/// Active tab in the Chern-Simons Anyon Interferometer Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChernSimonsTab {
    AnyonicInterferometer,
    FractionalStatistics,
    NonAbelianQuantumMemory,
    RealSpaceInterferometer,
    AuditTelemetry,
}

impl ChernSimonsTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::AnyonicInterferometer => "Anyonic Interferometer",
            Self::FractionalStatistics => "Fractional Statistics",
            Self::NonAbelianQuantumMemory => "Topological Memory",
            Self::RealSpaceInterferometer => "Interferometer Canvas",
            Self::AuditTelemetry => "Physics Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Phase 425.
pub struct ChernSimonsInterferometerDialog {
    pub is_open: bool,
    pub active_tab: ChernSimonsTab,

    // Interferometer Parameters
    pub anyon_kind: FractionalAnyonKind,
    pub qpc1_tunneling_prob: f64,
    pub qpc2_tunneling_prob: f64,
    pub interferometer_area_um2: f64,
    pub chiral_edge_velocity_ms: f64,
    pub bulk_anyon_number: usize,
    pub dephasing_rate_khz: f64,
    pub temperature_mk: f64,

    // Memory Parameters
    pub qubit_count: usize,
    pub cavity_quality_factor: f64,
    pub topological_gap_mhz: f64,
    pub storage_time_us: f64,
    pub readout_coupling_mhz: f64,

    // Cached Solver State
    pub processor: ChernSimonsMemoryProcessor,
    pub cached_spectrum: Vec<InterferometerTransmissionPoint>,
    pub cached_metrics: FractionalInterferenceMetrics,
    pub cached_decay: Vec<MemoryCoherenceDecayPoint>,
    pub cached_memory: TopologicalMemoryStateReport,
    pub cached_audit: ChernSimonsMemoryAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for ChernSimonsInterferometerDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl ChernSimonsInterferometerDialog {
    /// Fast cold-boot constructor with pre-computed baseline data (< 2.0 ms latency).
    pub fn new_fast() -> Self {
        let processor = ChernSimonsMemoryProcessor::default();
        let cached_spectrum = processor.interferometer.compute_interference_spectrum(100, 6.0);
        let cached_metrics = processor.interferometer.evaluate_interference_metrics();
        let cached_decay = processor.memory.compute_coherence_decay_curve(50, 400.0);
        let cached_memory = processor.memory.evaluate_memory_performance();
        let cached_audit = processor.audit_processor();

        Self {
            is_open: false,
            active_tab: ChernSimonsTab::AnyonicInterferometer,
            anyon_kind: processor.interferometer.params.anyon_kind,
            qpc1_tunneling_prob: processor.interferometer.params.qpc1_tunneling_prob,
            qpc2_tunneling_prob: processor.interferometer.params.qpc2_tunneling_prob,
            interferometer_area_um2: processor.interferometer.params.interferometer_area_um2,
            chiral_edge_velocity_ms: processor.interferometer.params.chiral_edge_velocity_ms,
            bulk_anyon_number: processor.interferometer.params.bulk_anyon_number,
            dephasing_rate_khz: processor.interferometer.params.dephasing_rate_khz,
            temperature_mk: processor.interferometer.params.temperature_mk,

            qubit_count: processor.memory.params.qubit_count,
            cavity_quality_factor: processor.memory.params.cavity_quality_factor,
            topological_gap_mhz: processor.memory.params.topological_gap_mhz,
            storage_time_us: processor.memory.params.storage_time_us,
            readout_coupling_mhz: processor.memory.params.readout_coupling_mhz,

            processor,
            cached_spectrum,
            cached_metrics,
            cached_decay,
            cached_memory,
            cached_audit,
            last_solve_time_us: 110.0,
        }
    }

    /// Recomputes interference spectrum, memory decay trajectories, and audit report.
    pub fn recompute(&mut self) {
        let start = crate::time_util::Instant::now();

        let inter_params = ChernSimonsInterferometerParams {
            anyon_kind: self.anyon_kind,
            qpc1_tunneling_prob: self.qpc1_tunneling_prob,
            qpc2_tunneling_prob: self.qpc2_tunneling_prob,
            interferometer_area_um2: self.interferometer_area_um2,
            chiral_edge_velocity_ms: self.chiral_edge_velocity_ms,
            bulk_anyon_number: self.bulk_anyon_number,
            dephasing_rate_khz: self.dephasing_rate_khz,
            temperature_mk: self.temperature_mk,
            ..Default::default()
        };

        let mem_params = AnyonicMemoryParams {
            qubit_count: self.qubit_count,
            cavity_quality_factor: self.cavity_quality_factor,
            topological_gap_mhz: self.topological_gap_mhz,
            storage_time_us: self.storage_time_us,
            readout_coupling_mhz: self.readout_coupling_mhz,
            cryogenic_temp_mk: self.temperature_mk,
        };

        self.processor = ChernSimonsMemoryProcessor::new(inter_params, mem_params);
        self.cached_spectrum = self.processor.interferometer.compute_interference_spectrum(100, 6.0);
        self.cached_metrics = self.processor.interferometer.evaluate_interference_metrics();
        self.cached_decay = self.processor.memory.compute_coherence_decay_curve(50, 400.0);
        self.cached_memory = self.processor.memory.evaluate_memory_performance();
        self.cached_audit = self.processor.audit_processor();

        self.last_solve_time_us = start.elapsed().as_micros() as f64;
    }

    /// Renders the complete dialog contents into the provided egui::Ui.
    pub fn render_contents(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(
                RichText::new("Chern-Simons Fractional Anyon Interferometer & Quantum Memory")
                    .strong()
                    .color(Color32::from_rgb(130, 220, 255)),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(RichText::new("Recompute Physics").strong()).clicked() {
                    self.recompute();
                }
            });
        });

        ui.separator();

        // Tab bar
        ui.horizontal(|ui| {
            let tabs = [
                ChernSimonsTab::AnyonicInterferometer,
                ChernSimonsTab::FractionalStatistics,
                ChernSimonsTab::NonAbelianQuantumMemory,
                ChernSimonsTab::RealSpaceInterferometer,
                ChernSimonsTab::AuditTelemetry,
            ];
            for tab in tabs {
                let selected = self.active_tab == tab;
                let text = if selected {
                    RichText::new(tab.label())
                        .strong()
                        .color(Color32::from_rgb(255, 215, 0))
                } else {
                    RichText::new(tab.label())
                };
                if ui.selectable_label(selected, text).clicked() {
                    self.active_tab = tab;
                }
            }
        });

        ui.separator();

        match self.active_tab {
            ChernSimonsTab::AnyonicInterferometer => self.render_interferometer_tab(ui),
            ChernSimonsTab::FractionalStatistics => self.render_statistics_tab(ui),
            ChernSimonsTab::NonAbelianQuantumMemory => self.render_memory_tab(ui),
            ChernSimonsTab::RealSpaceInterferometer => self.render_canvas_tab(ui),
            ChernSimonsTab::AuditTelemetry => self.render_audit_tab(ui),
        }
    }

    fn render_interferometer_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("QPC & Interferometer Controls").strong());
                let mut changed = false;

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.qpc1_tunneling_prob, 0.05..=0.80)
                            .text("QPC1 Tunneling |t1|^2"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.qpc2_tunneling_prob, 0.05..=0.80)
                            .text("QPC2 Tunneling |t2|^2"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.interferometer_area_um2, 10.0..=60.0)
                            .text("Cell Area (um^2)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.chiral_edge_velocity_ms, 500.0..=2500.0)
                            .text("Edge Velocity (m/s)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.dephasing_rate_khz, 5.0..=50.0)
                            .text("Dephasing Rate (kHz)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.temperature_mk, 5.0..=50.0)
                            .text("Cryo Temp (mK)"),
                    )
                    .changed();

                if changed {
                    self.recompute();
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Interference Metrics").strong());
                ui.label(format!(
                    "Visibility: {:.1}%",
                    self.cached_metrics.visibility * 100.0
                ));
                ui.label(format!(
                    "Contrast: {:.1} dB",
                    self.cached_metrics.interference_contrast_db
                ));
                ui.label(format!(
                    "Fractional Charge e*: {:.3}",
                    self.cached_metrics.effective_anyon_charge
                ));
                ui.label(format!(
                    "Dephasing Coherence Factor: {:.4}",
                    self.cached_metrics.dephasing_factor
                ));
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Fabry-Perot Interference Spectrum").strong());
                let trans_pts: PlotPoints = self
                    .cached_spectrum
                    .iter()
                    .map(|p| [p.flux_phi_over_phi0, p.transmission])
                    .collect();
                let cond_pts: PlotPoints = self
                    .cached_spectrum
                    .iter()
                    .map(|p| [p.flux_phi_over_phi0, p.longitudinal_conductance])
                    .collect();

                let line_trans = Line::new("Edge Transmission T", trans_pts)
                    .color(Color32::from_rgb(80, 200, 255))
                    .width(2.5);
                let line_cond = Line::new("Conductance G_xx (e^2/h)", cond_pts)
                    .color(Color32::from_rgb(255, 180, 50))
                    .width(2.5);

                let hline_target = HLine::new("85% Visibility Baseline", 0.85)
                    .color(Color32::from_rgb(0, 230, 100));

                Plot::new("chern_simons_spectrum_plot")
                    .height(280.0)
                    .x_axis_label("Synthetic Gauge Flux (Phi / Phi_0)")
                    .y_axis_label("Signal Amplitude")
                    .legend(egui_plot::Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.line(line_trans);
                        plot_ui.line(line_cond);
                        plot_ui.hline(hline_target);
                    });

                ui.label(
                    RichText::new("Cyan: Edge Transmission T | Gold: Longitudinal Conductance G_xx")
                        .color(Color32::LIGHT_GRAY),
                );
            });
        });
    }

    fn render_statistics_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Fractional Anyon Classification").strong());
                let anyons = [
                    FractionalAnyonKind::LaughlinOneThird,
                    FractionalAnyonKind::LaughlinOneFifth,
                    FractionalAnyonKind::MooreReadPfaffianFiveHalves,
                    FractionalAnyonKind::FibonacciAnyon,
                ];

                let mut changed = false;
                for a in anyons {
                    let selected = self.anyon_kind == a;
                    if ui.selectable_label(selected, a.name()).clicked() {
                        self.anyon_kind = a;
                        changed = true;
                    }
                }

                ui.add_space(8.0);
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.bulk_anyon_number, 0..=8)
                            .text("Enclosed Bulk Anyons (N)"),
                    )
                    .changed();

                if changed {
                    self.recompute();
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Statistical Parameters").strong());
                ui.label(format!("Fractional Charge e*: {:.3} e", self.anyon_kind.fractional_charge()));
                ui.label(format!(
                    "Exchange Phase Delta_phi: {:.3} rad ({:.1} deg)",
                    self.anyon_kind.statistical_exchange_phase_rad(),
                    self.anyon_kind.statistical_exchange_phase_rad().to_degrees()
                ));
                ui.label(format!("Quantum Dimension d: {:.3}", self.anyon_kind.quantum_dimension()));
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Statistical Exchange Phase Jump Extraction").strong());
                ui.add_space(4.0);

                let extracted_rad = self.cached_metrics.fractional_phase_jump_rad;
                let expected_rad = self.cached_metrics.expected_phase_jump_rad;
                let error_pct = self.cached_metrics.phase_error_relative * 100.0;

                ui.label(format!("Extracted Phase Jump: {:.4} rad ({:.2} deg)", extracted_rad, extracted_rad.to_degrees()));
                ui.label(format!("Theoretical Phase Jump: {:.4} rad ({:.2} deg)", expected_rad, expected_rad.to_degrees()));
                ui.label(format!("Extraction Error: {:.4}%", error_pct));

                ui.add_space(8.0);
                let (rect, _) = ui.allocate_exact_size(vec2(280.0, 30.0), Sense::hover());
                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 4.0, Color32::from_rgb(30, 40, 55));

                let bar_w = ((extracted_rad / (2.0 * std::f64::consts::PI)) as f32) * rect.width();
                painter.rect_filled(
                    Rect::from_min_size(rect.min, vec2(bar_w.min(rect.width()), rect.height())),
                    4.0,
                    Color32::from_rgb(0, 230, 150),
                );
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("Phase: {:.1} deg / 360 deg", extracted_rad.to_degrees()),
                    egui::FontId::proportional(12.0),
                    Color32::WHITE,
                );

                ui.add_space(12.0);
                ui.label(
                    RichText::new("Aharonov-Bohm vs Statistical Phase: Whenever an additional anyon enters the cell, the interference fringe pattern shifts discretely by Delta_phi.")
                        .italics()
                        .color(Color32::LIGHT_GRAY),
                );
            });
        });
    }

    fn render_memory_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Topological Memory Parameters").strong());
                let mut changed = false;

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.qubit_count, 1..=4)
                            .text("Logical Qubits"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.topological_gap_mhz, 10.0..=30.0)
                            .text("Topological Gap (MHz)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.storage_time_us, 50.0..=500.0)
                            .text("Storage Time (us)"),
                    )
                    .changed();

                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.readout_coupling_mhz, 1.0..=10.0)
                            .text("Readout Coupling (MHz)"),
                    )
                    .changed();

                if changed {
                    self.recompute();
                }

                ui.add_space(8.0);
                ui.label(RichText::new("Coherence Performance").strong());
                ui.label(format!(
                    "Topological Coherence T_2,topo: {:.1} us",
                    self.cached_memory.coherence_time_topo_us
                ));
                ui.label(format!(
                    "Bare Acoustic Coherence T_2,bare: {:.1} us",
                    self.cached_memory.coherence_time_bare_us
                ));
                ui.label(format!(
                    "Enhancement Ratio: {:.1}x",
                    self.cached_memory.coherence_enhancement_factor
                ));
                ui.label(format!(
                    "Storage/Retrieval Fidelity: {:.4}",
                    self.cached_memory.storage_retrieval_fidelity
                ));
                ui.label(format!(
                    "Parity Readout SNR: {:.1} dB",
                    self.cached_memory.parity_readout_snr_db
                ));
                ui.label(format!(
                    "Diabatic Leakage: {:.2e}",
                    self.cached_memory.diabatic_leakage_rate
                ));
                ui.label(format!("Purity: {:.4}", self.cached_memory.purity));
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label(RichText::new("Quantum State Fidelity Decay Trajectory").strong());
                let topo_pts: PlotPoints = self
                    .cached_decay
                    .iter()
                    .map(|p| [p.time_us, p.topological_fidelity])
                    .collect();
                let bare_pts: PlotPoints = self
                    .cached_decay
                    .iter()
                    .map(|p| [p.time_us, p.bare_acoustic_fidelity])
                    .collect();

                let line_topo = Line::new("Topological Memory F_topo(t)", topo_pts)
                    .color(Color32::from_rgb(0, 230, 150))
                    .width(2.5);
                let line_bare = Line::new("Bare Acoustic F_bare(t)", bare_pts)
                    .color(Color32::from_rgb(255, 90, 90))
                    .width(2.5);

                let hline_fid = HLine::new("Fidelity Threshold 0.90", 0.90)
                    .color(Color32::from_rgb(255, 215, 0));

                Plot::new("chern_simons_memory_decay_plot")
                    .height(280.0)
                    .x_axis_label("Hold Time (us)")
                    .y_axis_label("State Fidelity")
                    .legend(egui_plot::Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.line(line_topo);
                        plot_ui.line(line_bare);
                        plot_ui.hline(hline_fid);
                    });

                ui.label(
                    RichText::new("Green: Topologically Protected State | Red: Bare Resonator Dephasing")
                        .color(Color32::LIGHT_GRAY),
                );
            });
        });
    }

    fn render_canvas_tab(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Real-Space Chiral Acoustic Anyon Interferometer Canvas").strong());
        ui.add_space(4.0);

        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width().min(680.0), 320.0), Sense::hover());
        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 6.0, Color32::from_rgb(18, 24, 38));
        painter.rect_stroke(
            rect,
            6.0,
            Stroke::new(1.5, Color32::from_rgb(50, 75, 110)),
            StrokeKind::Inside,
        );

        let center = rect.center();
        let cell_w = 260.0;
        let cell_h = 140.0;
        let cell_rect = Rect::from_center_size(center, vec2(cell_w, cell_h));

        // Draw central interferometric cell area
        painter.rect_filled(cell_rect, 4.0, Color32::from_rgb(25, 38, 58));
        painter.rect_stroke(
            cell_rect,
            4.0,
            Stroke::new(2.0, Color32::from_rgb(70, 120, 180)),
            StrokeKind::Inside,
        );

        // Draw chiral edge channels (top: left to right, bottom: right to left)
        let stroke_edge = Stroke::new(3.0, Color32::from_rgb(0, 230, 200));
        let y_top = cell_rect.min.y;
        let y_bot = cell_rect.max.y;

        painter.line_segment([pos2(rect.min.x + 30.0, y_top), pos2(rect.max.x - 30.0, y_top)], stroke_edge);
        painter.arrow(pos2(rect.min.x + 80.0, y_top), vec2(60.0, 0.0), stroke_edge);
        painter.arrow(pos2(rect.max.x - 140.0, y_top), vec2(60.0, 0.0), stroke_edge);

        painter.line_segment([pos2(rect.min.x + 30.0, y_bot), pos2(rect.max.x - 30.0, y_bot)], stroke_edge);
        painter.arrow(pos2(rect.max.x - 80.0, y_bot), vec2(-60.0, 0.0), stroke_edge);
        painter.arrow(pos2(rect.min.x + 140.0, y_bot), vec2(-60.0, 0.0), stroke_edge);

        // Draw two Quantum Point Contacts (QPC1 and QPC2)
        let qpc1_x = cell_rect.min.x + 40.0;
        let qpc2_x = cell_rect.max.x - 40.0;

        for (x, label) in [(qpc1_x, "QPC 1"), (qpc2_x, "QPC 2")] {
            let col_qpc = Color32::from_rgb(255, 180, 50);
            // Upper constriction gate
            painter.rect_filled(Rect::from_center_size(pos2(x, y_top), vec2(14.0, 24.0)), 2.0, col_qpc);
            // Lower constriction gate
            painter.rect_filled(Rect::from_center_size(pos2(x, y_bot), vec2(14.0, 24.0)), 2.0, col_qpc);
            // Tunneling dashed line between gates
            painter.line_segment(
                [pos2(x, y_top + 12.0), pos2(x, y_bot - 12.0)],
                Stroke::new(1.5, Color32::from_rgb(255, 215, 0)),
            );
            painter.text(
                pos2(x, y_top - 16.0),
                egui::Align2::CENTER_BOTTOM,
                label,
                egui::FontId::proportional(12.0),
                Color32::WHITE,
            );
        }

        // Draw synthetic flux vortex at the center
        painter.circle_stroke(center, 24.0, Stroke::new(1.5, Color32::from_rgb(100, 160, 255)));
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            "Phi (Flux)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(180, 220, 255),
        );

        // Draw trapped bulk anyons inside the cell
        let n_anyons = self.bulk_anyon_number.min(6);
        for i in 0..n_anyons {
            let angle = (i as f32) / (n_anyons as f32) * std::f32::consts::TAU;
            let ax = center.x + 55.0 * angle.cos();
            let ay = center.y + 35.0 * angle.sin();
            painter.circle_filled(pos2(ax, ay), 9.0, Color32::from_rgb(255, 80, 80));
            painter.circle_stroke(pos2(ax, ay), 12.0, Stroke::new(1.0, Color32::from_rgb(255, 180, 180)));
        }

        painter.text(
            pos2(center.x, cell_rect.max.y + 30.0),
            egui::Align2::CENTER_TOP,
            format!("Fabry-Perot Loop: Area = {:.1} um^2 | {} Enclosed Bulk Anyons", self.interferometer_area_um2, self.bulk_anyon_number),
            egui::FontId::proportional(12.0),
            Color32::from_rgb(180, 220, 255),
        );
    }

    fn render_audit_tab(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("10-Point Physics & Quantum Verification Checklist").strong());
        ui.add_space(4.0);

        let report = &self.cached_audit;

        let items = [
            (
                report.pass_k_matrix_quantization,
                "1. Chern-Simons K-matrix topological quantization",
            ),
            (
                report.pass_statistical_phase_accuracy,
                "2. Fractional statistical exchange phase accuracy (error < 1.0%)",
            ),
            (
                report.pass_interference_visibility,
                "3. Fabry-Perot anyon interference visibility >= 85%",
            ),
            (
                report.pass_interference_contrast,
                "4. Interference contrast >= 18.0 dB",
            ),
            (
                report.pass_anyon_charge_fraction,
                "5. Anyonic fractional charge quantization e* matches theory",
            ),
            (
                report.pass_topological_coherence_time,
                "6. Topologically protected coherence time T_2,topo >= 250 us",
            ),
            (
                report.pass_coherence_enhancement,
                "7. Coherence enhancement over bare acoustics >= 20.0x",
            ),
            (
                report.pass_storage_fidelity,
                "8. Quantum memory state storage and retrieval fidelity >= 0.995",
            ),
            (
                report.pass_readout_snr,
                "9. Non-destructive parity readout SNR >= 18.0 dB",
            ),
            (
                report.pass_diabatic_suppression,
                "10. Diabatic state leakage rate < 1e-4",
            ),
        ];

        for (pass, label) in items {
            ui.horizontal(|ui| {
                if pass {
                    ui.label(RichText::new("[PASS]").strong().color(Color32::from_rgb(0, 230, 100)));
                } else {
                    ui.label(RichText::new("[FAIL]").strong().color(Color32::from_rgb(255, 60, 60)));
                }
                ui.label(label);
            });
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.label(format!("Audit Score: {} / 10", report.pass_count));
            if report.all_passed {
                ui.label(
                    RichText::new("ALL PHYSICS CRITERIA VERIFIED")
                        .strong()
                        .color(Color32::from_rgb(0, 230, 100)),
                );
            }
            ui.label(format!("(Solve Time: {:.1} us)", self.last_solve_time_us));
        });
    }

    /// Renders the modal dialog window if open.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut open = self.is_open;
        Window::new("Chern-Simons Fractional Anyon Interferometer & Quantum Memory")
            .open(&mut open)
            .default_size(vec2(780.0, 520.0))
            .show(ctx, |ui| {
                self.render_contents(ui);
            });
        self.is_open = open;
    }

    /// Alias for showing the modal dialog.
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }
}
