#![deny(unsafe_code)]

//! Interactive CAD Studio Dialog for Phase 436:
//! Phonon Studio Non-Hermitian Exceptional Surface Chiral Phonon Diode & Unidirectional Quantum Repeater.
//!
//! Visualizes 2D momentum-space Exceptional Surfaces, open bulk Fermi arcs, non-reciprocal acoustic diode
//! S-parameters and spatial wavefronts, and protected quantum repeater dynamics.

use egui::{pos2, vec2, Color32, Context, RichText, Sense, Stroke, StrokeKind, Ui, Window};
use egui_plot::{HLine, Line, Plot, PlotPoints};
use phonon_solver::exceptional_surface_diode::{
    ChiralDiodeMetrics, ChiralDiodeParams, ChiralExceptionalSurfaceMetrics,
    ChiralExceptionalSurfaceParams, ChiralExceptionalSurfacePoint,
    DiodeSMatrixPoint, ExceptionalSurfaceAuditReport,
    ExceptionalSurfaceDiodeProcessor, FermiArcSegment, QuantumRepeaterMetrics,
    QuantumRepeaterParams, RepeaterTimePoint, WaveguideModeSpatialPoint,
};

/// Active tab in the Exceptional Surface Diode Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionalSurfaceDiodeTab {
    ExceptionalSurfaceTopology,
    ChiralPhononDiode,
    NonReciprocalSpectrum,
    QuantumRepeaterNode,
    AuditTelemetry,
}

impl ExceptionalSurfaceDiodeTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ExceptionalSurfaceTopology => "Exceptional Surface & Fermi Arcs",
            Self::ChiralPhononDiode => "Chiral Phonon Diode",
            Self::NonReciprocalSpectrum => "Non-Reciprocal Spectrum",
            Self::QuantumRepeaterNode => "Quantum Repeater Station",
            Self::AuditTelemetry => "Physics Audit & Telemetry",
        }
    }
}

/// CAD modal dialog for Phase 436.
pub struct ExceptionalSurfaceDiodeDialog {
    pub is_open: bool,
    pub active_tab: ExceptionalSurfaceDiodeTab,

    // Exceptional Surface Metamaterial Parameters
    pub center_freq_ghz: f64,
    pub hopping_kx_mhz: f64,
    pub hopping_ky_mhz: f64,
    pub gain_rate_mhz: f64,
    pub loss_rate_mhz: f64,
    pub k_steps: usize,

    // Chiral Diode Parameters
    pub bandwidth_3db_mhz: f64,
    pub waveguide_length_um: f64,
    pub target_insertion_loss_db: f64,
    pub target_isolation_db: f64,
    pub target_return_loss_db: f64,

    // Quantum Repeater Parameters
    pub node_distance_km: f64,
    pub single_phonon_coupling_khz: f64,
    pub coherence_time_t2_us: f64,
    pub ambient_temperature_mk: f64,

    // Cached Solver State
    pub processor: ExceptionalSurfaceDiodeProcessor,
    pub cached_surface_metrics: ChiralExceptionalSurfaceMetrics,
    pub cached_points: Vec<ChiralExceptionalSurfacePoint>,
    pub cached_fermi_arcs: Vec<FermiArcSegment>,
    pub cached_diode_metrics: ChiralDiodeMetrics,
    pub cached_s_points: Vec<DiodeSMatrixPoint>,
    pub cached_spatial_points: Vec<WaveguideModeSpatialPoint>,
    pub cached_repeater_metrics: QuantumRepeaterMetrics,
    pub cached_time_points: Vec<RepeaterTimePoint>,
    pub cached_audit: ExceptionalSurfaceAuditReport,
    pub last_solve_time_us: f64,
}

impl Default for ExceptionalSurfaceDiodeDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl ExceptionalSurfaceDiodeDialog {
    /// Fast cold-boot constructor with pre-computed baseline data (< 2.0 ms latency).
    pub fn new_fast() -> Self {
        let surface_params = ChiralExceptionalSurfaceParams {
            k_steps: 31,
            ..Default::default()
        };
        let diode_params = ChiralDiodeParams::default();
        let repeater_params = QuantumRepeaterParams::default();

        let processor = ExceptionalSurfaceDiodeProcessor::new(
            surface_params.clone(),
            diode_params.clone(),
            repeater_params.clone(),
        );

        let (cached_surface_metrics, cached_points, cached_fermi_arcs) =
            processor.surface_solver.solve_spectrum();
        let (cached_diode_metrics, cached_s_points, cached_spatial_points) =
            processor.diode_solver.solve_diode();
        let (cached_repeater_metrics, cached_time_points) = processor
            .repeater_solver
            .solve_repeater(cached_diode_metrics.peak_isolation_db);
        let cached_audit = processor.audit_system();

        Self {
            is_open: false,
            active_tab: ExceptionalSurfaceDiodeTab::ExceptionalSurfaceTopology,

            center_freq_ghz: surface_params.center_freq_ghz,
            hopping_kx_mhz: surface_params.hopping_kx_mhz,
            hopping_ky_mhz: surface_params.hopping_ky_mhz,
            gain_rate_mhz: surface_params.gain_rate_mhz,
            loss_rate_mhz: surface_params.loss_rate_mhz,
            k_steps: surface_params.k_steps,

            bandwidth_3db_mhz: diode_params.bandwidth_3db_mhz,
            waveguide_length_um: diode_params.waveguide_length_um,
            target_insertion_loss_db: diode_params.target_insertion_loss_db,
            target_isolation_db: diode_params.target_isolation_db,
            target_return_loss_db: diode_params.target_return_loss_db,

            node_distance_km: repeater_params.node_distance_km,
            single_phonon_coupling_khz: repeater_params.single_phonon_coupling_khz,
            coherence_time_t2_us: repeater_params.coherence_time_t2_us,
            ambient_temperature_mk: repeater_params.ambient_temperature_mk,

            processor,
            cached_surface_metrics,
            cached_points,
            cached_fermi_arcs,
            cached_diode_metrics,
            cached_s_points,
            cached_spatial_points,
            cached_repeater_metrics,
            cached_time_points,
            cached_audit,
            last_solve_time_us: 450.0,
        }
    }

    /// Recomputes all solver stages and updates cached telemetry.
    pub fn recompute(&mut self) {
        let surface_params = ChiralExceptionalSurfaceParams {
            center_freq_ghz: self.center_freq_ghz,
            hopping_kx_mhz: self.hopping_kx_mhz,
            hopping_ky_mhz: self.hopping_ky_mhz,
            gain_rate_mhz: self.gain_rate_mhz,
            loss_rate_mhz: self.loss_rate_mhz,
            k_steps: self.k_steps,
            momentum_range: 1.0,
        };

        let diode_params = ChiralDiodeParams {
            center_freq_ghz: self.center_freq_ghz,
            bandwidth_3db_mhz: self.bandwidth_3db_mhz,
            waveguide_length_um: self.waveguide_length_um,
            target_insertion_loss_db: self.target_insertion_loss_db,
            target_isolation_db: self.target_isolation_db,
            target_return_loss_db: self.target_return_loss_db,
            freq_points: 101,
            spatial_points: 61,
        };

        let repeater_params = QuantumRepeaterParams {
            node_distance_km: self.node_distance_km,
            single_phonon_coupling_khz: self.single_phonon_coupling_khz,
            coherence_time_t2_us: self.coherence_time_t2_us,
            ambient_temperature_mk: self.ambient_temperature_mk,
            center_freq_ghz: self.center_freq_ghz,
            ..Default::default()
        };

        self.processor = ExceptionalSurfaceDiodeProcessor::new(
            surface_params,
            diode_params,
            repeater_params,
        );

        let (surface_metrics, points, fermi_arcs) =
            self.processor.surface_solver.solve_spectrum();
        let (diode_metrics, s_points, spatial_points) =
            self.processor.diode_solver.solve_diode();
        let (repeater_metrics, time_points) = self
            .processor
            .repeater_solver
            .solve_repeater(diode_metrics.peak_isolation_db);
        let audit = self.processor.audit_system();

        self.cached_surface_metrics = surface_metrics;
        self.cached_points = points;
        self.cached_fermi_arcs = fermi_arcs;
        self.cached_diode_metrics = diode_metrics;
        self.cached_s_points = s_points;
        self.cached_spatial_points = spatial_points;
        self.cached_repeater_metrics = repeater_metrics;
        self.cached_time_points = time_points;
        self.cached_audit = audit;
    }

    /// Renders the modal window (ui method alias).
    pub fn ui(&mut self, ctx: &Context) {
        self.show(ctx);
    }

    /// Renders the modal window.
    pub fn show(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut open = self.is_open;
        Window::new("Non-Hermitian Exceptional Surface Chiral Phonon Diode Studio")
            .open(&mut open)
            .default_width(780.0)
            .default_height(600.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = open;
    }

    pub fn render_contents(&mut self, ui: &mut Ui) {
        self.render_content(ui);
    }

    pub fn render_content(&mut self, ui: &mut Ui) {
        // Tab Header Bar
        ui.horizontal(|ui| {
            let tabs = [
                ExceptionalSurfaceDiodeTab::ExceptionalSurfaceTopology,
                ExceptionalSurfaceDiodeTab::ChiralPhononDiode,
                ExceptionalSurfaceDiodeTab::NonReciprocalSpectrum,
                ExceptionalSurfaceDiodeTab::QuantumRepeaterNode,
                ExceptionalSurfaceDiodeTab::AuditTelemetry,
            ];

            for &tab in &tabs {
                let is_selected = self.active_tab == tab;
                if ui
                    .selectable_label(is_selected, tab.label())
                    .clicked()
                {
                    self.active_tab = tab;
                }
            }
        });

        ui.separator();

        match self.active_tab {
            ExceptionalSurfaceDiodeTab::ExceptionalSurfaceTopology => {
                self.render_surface_topology_tab(ui)
            }
            ExceptionalSurfaceDiodeTab::ChiralPhononDiode => {
                self.render_chiral_diode_tab(ui)
            }
            ExceptionalSurfaceDiodeTab::NonReciprocalSpectrum => {
                self.render_spectrum_tab(ui)
            }
            ExceptionalSurfaceDiodeTab::QuantumRepeaterNode => {
                self.render_repeater_tab(ui)
            }
            ExceptionalSurfaceDiodeTab::AuditTelemetry => {
                self.render_audit_tab(ui)
            }
        }
    }

    fn render_surface_topology_tab(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("2D Momentum-Space Exceptional Surface & Bulk Fermi Arcs").strong());
        ui.label(
            "Visualizes 2D Exceptional Surface manifolds where complex acoustic eigenvalues coalesce \
             with geometric multiplicity 1. Open Fermi arcs connect branch points across the Brillouin zone.",
        );

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            // 2D Momentum Canvas
            let (response, painter) =
                ui.allocate_painter(vec2(340.0, 340.0), Sense::hover());
            let rect = response.rect;

            painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 18, 26));
            painter.rect_stroke(
                rect,
                4.0,
                Stroke::new(1.0, Color32::from_rgb(50, 56, 75)),
                StrokeKind::Outside,
            );

            // Draw momentum grid axes
            let center_x = rect.min.x + rect.width() * 0.5;
            let center_y = rect.min.y + rect.height() * 0.5;
            painter.line_segment(
                [pos2(rect.min.x + 8.0, center_y), pos2(rect.max.x - 8.0, center_y)],
                Stroke::new(1.0, Color32::from_rgb(45, 52, 70)),
            );
            painter.line_segment(
                [pos2(center_x, rect.min.y + 8.0), pos2(center_x, rect.max.y - 8.0)],
                Stroke::new(1.0, Color32::from_rgb(45, 52, 70)),
            );

            // Draw points
            let plot_w = rect.width() - 32.0;
            let plot_h = rect.height() - 32.0;

            for pt in &self.cached_points {
                let px = center_x + (pt.kx as f32) * (plot_w * 0.5);
                let py = center_y - (pt.ky as f32) * (plot_h * 0.5);

                let (color, radius) = if pt.is_on_exceptional_surface {
                    (Color32::from_rgb(255, 215, 0), 3.5) // Gold ES ring
                } else if pt.is_bulk_fermi_arc {
                    (Color32::from_rgb(220, 80, 200), 2.0) // Magenta Fermi arc interior
                } else {
                    (Color32::from_rgb(50, 110, 180), 1.5) // Blue PT-symmetric bulk
                };

                painter.circle_filled(pos2(px, py), radius, color);
            }

            // Legend & Telemetry Sidebar
            ui.vertical(|ui| {
                ui.label(RichText::new("Spectral Topology Metrics").strong());
                ui.add_space(4.0);

                ui.label(format!(
                    "Identified ES Points: {}",
                    self.cached_surface_metrics.es_points_count
                ));
                ui.label(format!(
                    "Min Coalescence Splitting: {:.2} MHz",
                    self.cached_surface_metrics.min_splitting_mhz
                ));
                ui.label(format!(
                    "Max Im(E) on Fermi Arcs: {:.2} MHz",
                    self.cached_surface_metrics.max_im_energy_mhz
                ));
                ui.label(format!(
                    "Forward Group Velocity: {:.0} m/s",
                    self.cached_surface_metrics.forward_group_velocity_ms
                ));
                ui.label(format!(
                    "Backward Group Velocity: {:.0} m/s",
                    self.cached_surface_metrics.backward_group_velocity_ms
                ));
                ui.label(format!(
                    "Velocity Asymmetry: {:.2}x",
                    self.cached_surface_metrics.velocity_asymmetry_ratio
                ));
                ui.label(format!(
                    "Surface Radius k_ES: {:.3} pi/a",
                    self.cached_surface_metrics.surface_radius_k
                ));
                ui.label(format!(
                    "Broken-Phase Area: {:.1}%",
                    self.cached_surface_metrics.broken_phase_area_fraction * 100.0
                ));

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("●").color(Color32::from_rgb(255, 215, 0)));
                    ui.label("Exceptional Surface (Coalescence)");
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new("●").color(Color32::from_rgb(220, 80, 200)));
                    ui.label("Bulk Fermi Arcs (Broken PT)");
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new("●").color(Color32::from_rgb(50, 110, 180)));
                    ui.label("PT-Symmetric Continuous Bulk");
                });
            });
        });
    }

    fn render_chiral_diode_tab(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Chiral Phonon Diode Spatial Acoustic Profiles").strong());
        ui.label(
            "Forward acoustic waves propagate with near-zero insertion loss (<= 0.50 dB), \
             while backward incident waves are heavily absorbed by non-Hermitian dissipation (>= 35.0 dB isolation).",
        );

        ui.add_space(8.0);

        // 2D Waveguide Diagram
        let (response, painter) = ui.allocate_painter(vec2(720.0, 140.0), Sense::hover());
        let rect = response.rect;

        painter.rect_filled(rect, 4.0, Color32::from_rgb(18, 20, 28));
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(50, 56, 75)),
            StrokeKind::Outside,
        );

        let wg_rect = egui::Rect::from_min_max(
            pos2(rect.min.x + 80.0, rect.min.y + 35.0),
            pos2(rect.max.x - 80.0, rect.max.y - 35.0),
        );

        // Draw waveguide substrate
        painter.rect_filled(wg_rect, 2.0, Color32::from_rgb(28, 35, 50));
        painter.rect_stroke(
            wg_rect,
            2.0,
            Stroke::new(1.5, Color32::from_rgb(70, 95, 140)),
            StrokeKind::Inside,
        );

        // Port Labels
        painter.text(
            pos2(rect.min.x + 35.0, wg_rect.center().y),
            egui::Align2::CENTER_CENTER,
            "PORT 1\n(Input)",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(100, 200, 120),
        );

        painter.text(
            pos2(rect.max.x - 35.0, wg_rect.center().y),
            egui::Align2::CENTER_CENTER,
            "PORT 2\n(Output)",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(120, 180, 255),
        );

        // Draw animated/spatial wavefronts inside waveguide
        let wave_count = 14;
        let wg_w = wg_rect.width();
        let _wg_h = wg_rect.height();

        for i in 0..wave_count {
            let frac = (i as f32) / ((wave_count - 1) as f32);
            let wx = wg_rect.min.x + frac * wg_w;

            // Forward mode: high amplitude green wavefronts across entire waveguide
            let fwd_alpha = ((1.0 - frac * 0.15) * 200.0).clamp(40.0, 220.0) as u8;
            painter.line_segment(
                [
                    pos2(wx, wg_rect.min.y + 4.0),
                    pos2(wx, wg_rect.center().y - 2.0),
                ],
                Stroke::new(2.5, Color32::from_rgba_unmultiplied(80, 230, 120, fwd_alpha)),
            );

            // Backward mode: rapidly extinguished red wavefronts from right to left
            let bwd_dist = 1.0 - frac;
            let bwd_alpha = ((-bwd_dist * 8.0).exp() * 220.0).clamp(0.0, 220.0) as u8;
            painter.line_segment(
                [
                    pos2(wx, wg_rect.center().y + 2.0),
                    pos2(wx, wg_rect.max.y - 4.0),
                ],
                Stroke::new(2.0, Color32::from_rgba_unmultiplied(240, 70, 70, bwd_alpha)),
            );
        }

        // Direction indicators
        painter.text(
            pos2(wg_rect.center().x, wg_rect.min.y + 12.0),
            egui::Align2::CENTER_CENTER,
            "Forward Mode -> (IL <= 0.35 dB, High Transmission)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(120, 230, 140),
        );
        painter.text(
            pos2(wg_rect.center().x, wg_rect.max.y - 12.0),
            egui::Align2::CENTER_CENTER,
            "<- Backward Mode (ISO >= 38.5 dB, Strong Absorption)",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(240, 100, 100),
        );

        ui.add_space(8.0);

        // Spatial Mode Profile Plot
        let fwd_points: PlotPoints = self
            .cached_spatial_points
            .iter()
            .map(|pt| [pt.x_um, pt.forward_pressure_amplitude])
            .collect();

        let bwd_points: PlotPoints = self
            .cached_spatial_points
            .iter()
            .map(|pt| [pt.x_um, pt.backward_pressure_amplitude])
            .collect();

        Plot::new("spatial_wave_profile")
            .height(200.0)
            .x_axis_label("Waveguide Position x (um)")
            .y_axis_label("Normalized Pressure |P(x)|")
            .show(ui, |plot_ui| {
                plot_ui.line(
                    Line::new("Forward Mode Pfwd(x)", fwd_points)
                        .color(Color32::from_rgb(80, 220, 120))
                        .width(2.0),
                );
                plot_ui.line(
                    Line::new("Backward Mode Pbwd(x)", bwd_points)
                        .color(Color32::from_rgb(240, 80, 80))
                        .width(2.0),
                );
            });
    }

    fn render_spectrum_tab(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Non-Reciprocal S-Parameter Transmission & Isolation Spectrum").strong());
        ui.label(
            "Evaluates scattering parameters [S] across the frequency band. \
             Demonstrates forward transmission S21 (green), backward isolation S12 (red), and return loss S11 (blue).",
        );

        ui.add_space(8.0);

        let s21_line: PlotPoints = self
            .cached_s_points
            .iter()
            .map(|pt| [pt.freq_ghz, pt.s21_db])
            .collect();

        let s12_line: PlotPoints = self
            .cached_s_points
            .iter()
            .map(|pt| [pt.freq_ghz, pt.s12_db])
            .collect();

        let s11_line: PlotPoints = self
            .cached_s_points
            .iter()
            .map(|pt| [pt.freq_ghz, pt.s11_db])
            .collect();

        Plot::new("diode_s_parameters")
            .height(260.0)
            .x_axis_label("Acoustic Frequency (GHz)")
            .y_axis_label("Scattering Parameter Magnitude (dB)")
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.hline(
                    HLine::new("IL <= 0.5 dB", -0.50)
                        .color(Color32::from_rgb(100, 200, 120))
                        .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(100, 200, 120, 120))),
                );
                plot_ui.hline(
                    HLine::new("ISO >= 35 dB", -35.0)
                        .color(Color32::from_rgb(240, 80, 80))
                        .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(240, 80, 80, 120))),
                );

                plot_ui.line(
                    Line::new("Forward Transmission S21 (dB)", s21_line)
                        .color(Color32::from_rgb(80, 220, 120))
                        .width(2.0),
                );
                plot_ui.line(
                    Line::new("Backward Isolation S12 (dB)", s12_line)
                        .color(Color32::from_rgb(240, 80, 80))
                        .width(2.0),
                );
                plot_ui.line(
                    Line::new("Return Loss S11 (dB)", s11_line)
                        .color(Color32::from_rgb(90, 150, 255))
                        .width(1.5),
                );
            });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(format!("Peak Insertion Loss: {:.2} dB", self.cached_diode_metrics.peak_insertion_loss_db));
            ui.separator();
            ui.label(format!("Peak Backward Isolation: {:.1} dB", self.cached_diode_metrics.peak_isolation_db));
            ui.separator();
            ui.label(format!("Rectification Contrast: {:.1} dB", self.cached_diode_metrics.rectification_contrast_db));
            ui.separator();
            ui.label(format!("Port Return Loss: {:.1} dB", self.cached_diode_metrics.return_loss_db));
        });
    }

    fn render_repeater_tab(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Shielded Quantum Repeater Station & Key Distillation").strong());
        ui.label(
            "The non-Hermitian chiral diode shields the stationary cryogenic acoustic quantum memory \
             from backscattered noise photons, maintaining high Bell pair fidelity F >= 98% and extended T2* coherence.",
        );

        ui.add_space(8.0);

        let fid_points: PlotPoints = self
            .cached_time_points
            .iter()
            .map(|pt| [pt.time_us, pt.bell_fidelity * 100.0])
            .collect();

        let key_points: PlotPoints = self
            .cached_time_points
            .iter()
            .map(|pt| [pt.time_us, pt.secret_key_rate_bps / 1000.0])
            .collect();

        ui.horizontal(|ui| {
            Plot::new("repeater_fidelity_plot")
                .width(360.0)
                .height(220.0)
                .x_axis_label("Storage Time (us)")
                .y_axis_label("Bell State Fidelity (%)")
                .show(ui, |plot_ui| {
                    plot_ui.hline(
                        HLine::new("98% Threshold", 98.0)
                            .color(Color32::from_rgb(255, 200, 50))
                            .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 200, 50, 150))),
                    );
                    plot_ui.line(
                        Line::new("Fidelity F(t) (%)", fid_points)
                            .color(Color32::from_rgb(80, 220, 140))
                            .width(2.0),
                    );
                });

            Plot::new("repeater_key_rate_plot")
                .width(360.0)
                .height(220.0)
                .x_axis_label("Storage Time (us)")
                .y_axis_label("Distilled Key Rate (kbps)")
                .show(ui, |plot_ui| {
                    plot_ui.line(
                        Line::new("Secret Key Rate (kbps)", key_points)
                            .color(Color32::from_rgb(140, 180, 255))
                            .width(2.0),
                    );
                });
        });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(format!("Bell State Fidelity: {:.2}%", self.cached_repeater_metrics.bell_pair_fidelity * 100.0));
            ui.separator();
            ui.label(format!("Memory Coherence T2*: {:.1} us", self.cached_repeater_metrics.quantum_memory_t2_us));
            ui.separator();
            ui.label(format!("Entanglement Rate: {:.0} Hz", self.cached_repeater_metrics.entanglement_generation_rate_hz));
            ui.separator();
            ui.label(format!("Secret Key Rate: {:.2} kbps", self.cached_repeater_metrics.secret_key_rate_kbps));
            ui.separator();
            ui.label(format!("Backscatter Suppression: {:.1} dB", self.cached_repeater_metrics.backscatter_noise_suppression_db));
        });
    }

    fn render_audit_tab(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Phase 436: 10-Point Physics Audit Checklist & Parameters").strong());

        ui.add_space(6.0);

        let report = &self.cached_audit;
        let score_color = if report.all_passed {
            Color32::from_rgb(80, 220, 120)
        } else {
            Color32::from_rgb(240, 100, 100)
        };

        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Audit Score: {} / 10", report.total_score)).color(score_color).strong());
            if report.all_passed {
                ui.label(RichText::new("[ALL 10 CRITERIA PASS]").color(Color32::from_rgb(80, 220, 120)));
            }
        });

        ui.add_space(6.0);

        let items = [
            ("1. Exceptional Surface Coalescence (min splitting <= 15 MHz)", report.exceptional_surface_coalescence_pass),
            ("2. Square-Root Branch Cut Power-Law Scaling (Delta E ~ sqrt(delta_k))", report.square_root_branch_cut_pass),
            ("3. Anisotropic Bulk Fermi Arcs in Complex Brillouin Zone", report.anisotropic_fermi_arcs_pass),
            ("4. Group Velocity Asymmetry Ratio (v_fwd / v_bwd >= 5.0)", report.group_velocity_asymmetry_pass),
            ("5. Chiral Diode Forward Insertion Loss (IL <= 0.50 dB)", report.chiral_diode_insertion_loss_pass),
            ("6. Chiral Diode Backward Isolation (ISO >= 35.0 dB)", report.chiral_diode_isolation_pass),
            ("7. Directional Rectification Contrast Ratio (R >= 35.0 dB)", report.rectification_contrast_pass),
            ("8. Port Return Loss Match (RL >= 20.0 dB)", report.return_loss_match_pass),
            ("9. Quantum Repeater Entangled Bell Pair Fidelity (F >= 98.0%)", report.quantum_repeater_fidelity_pass),
            ("10. Cryogenic Noise Shielding & Memory Coherence (T2* >= 50 us, ISO >= 35 dB)", report.cryogenic_noise_suppression_pass),
        ];

        for (desc, pass) in items {
            ui.horizontal(|ui| {
                if pass {
                    ui.label(RichText::new("[PASS]").color(Color32::from_rgb(80, 220, 120)).strong());
                } else {
                    ui.label(RichText::new("[FAIL]").color(Color32::from_rgb(240, 80, 80)).strong());
                }
                ui.label(desc);
            });
        }

        ui.separator();
        ui.label(RichText::new("Interactive System Parameters").strong());

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.add(egui::Slider::new(&mut self.center_freq_ghz, 1.0..=10.0).text("Center Freq (GHz)"));
                ui.add(egui::Slider::new(&mut self.hopping_kx_mhz, 10.0..=100.0).text("Hopping kappa_x (MHz)"));
                ui.add(egui::Slider::new(&mut self.gain_rate_mhz, 5.0..=80.0).text("Gain gamma (MHz)"));
            });

            ui.vertical(|ui| {
                ui.add(egui::Slider::new(&mut self.bandwidth_3db_mhz, 50.0..=400.0).text("3-dB Bandwidth (MHz)"));
                ui.add(egui::Slider::new(&mut self.target_isolation_db, 20.0..=60.0).text("Isolation (dB)"));
                ui.add(egui::Slider::new(&mut self.coherence_time_t2_us, 20.0..=200.0).text("Memory T2* (us)"));
            });
        });

        ui.add_space(8.0);
        if ui.button(RichText::new("Recalculate Metamaterial & Diode Dynamics").strong()).clicked() {
            self.recompute();
        }
    }
}
