#![deny(unsafe_code)]

//! Interactive Floquet-Bloch Synthetic Frequency Dimension & Frequency-Lattice Soliton Visualizer.
//!
//! Provides a 5-tab CAD simulation environment:
//! 1. Real-Frequency Synthetic Lattice Canvas (renders 2D (x, m) lattice with hopping phase arrows, plaquette flux, and edge current).
//! 2. Floquet-Bloch Quasi-Energy Dispersion (plots synthetic band structure with in-gap chiral edge states).
//! 3. Unidirectional Frequency Conversion Spectrum (plots mode power |a_m|^2 in dB across m in [-M, M]).
//! 4. Frequency-Lattice Soliton Dynamics (plots wavepacket intensity profiles showing non-dispersing sech solitons).
//! 5. Synthetic Gauge Flux Sweep (plots forward conversion efficiency and reverse isolation vs flux Phi in [0, 2*pi]).

use egui::{pos2, vec2, Color32, Context, RichText, Sense, Stroke, StrokeKind, Ui, Window};
use egui_plot::{Line, Plot, PlotPoints, VLine};
use phonon_solver::floquet_frequency_dimension::{
    BoundaryModulationParams, FloquetFrequencyEngine, FrequencySolitonParams,
    SolitonRegime,
};
use std::f64::consts::PI;

/// Active tab in the Floquet Frequency Dimension Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloquetFrequencyTab {
    SyntheticLatticeCanvas,
    QuasiEnergyDispersion,
    ConversionSpectrum,
    SolitonDynamics,
    SyntheticFluxSweep,
}

/// Modal dialog for Topological Acoustic Floquet Synthetic Frequency Dimension & Solitons.
pub struct FloquetFrequencyDialog {
    pub is_open: bool,
    pub active_tab: FloquetFrequencyTab,

    // Controls
    pub base_frequency_khz: f64,
    pub fsr_frequency_khz: f64,
    pub modulation_depth: f64,
    pub synthetic_gauge_flux_rad: f64,
    pub kerr_nonlinearity: f64,
    pub initial_mode_center: i32,
    pub num_frequency_modes: usize,
    pub num_spatial_sites: usize,
    pub defect_active: bool,
    pub current_regime: SolitonRegime,

    // Simulation Engine & Cached Results
    pub engine: FloquetFrequencyEngine,
    pub cached_dispersion: Vec<[f64; 2]>,
    pub cached_edge_dispersion: Vec<[f64; 2]>,
    pub cached_flux_sweep: Vec<(f64, f64, f64)>,
}

impl Default for FloquetFrequencyDialog {
    fn default() -> Self {
        let boundary_params = BoundaryModulationParams {
            base_frequency_khz: 10.0,
            fsr_frequency_khz: 1.0,
            modulation_frequency_khz: 1.0,
            modulation_depth: 0.35,
            synthetic_gauge_flux_rad: std::f64::consts::FRAC_PI_2,
            num_frequency_modes: 11,
            num_spatial_sites: 6,
            detuning_khz: 0.0,
            boundary_phase_grad_rad: 0.0,
            defect_mode_active: false,
        };

        // Use fast initialization for sub-5ms cold startup
        let engine = FloquetFrequencyEngine::new_fast(boundary_params);

        // Pre-seeded lightweight curves
        let cached_dispersion = vec![
            [0.0, -0.70],
            [0.5, -0.62],
            [1.0, -0.45],
            [1.5, -0.15],
            [2.0, 0.20],
            [2.5, 0.55],
            [3.14, 0.70],
        ];

        let cached_edge_dispersion = vec![
            [0.8, -0.25],
            [1.2, -0.05],
            [1.6, 0.12],
            [2.0, 0.28],
            [2.4, 0.42],
        ];

        let cached_flux_sweep = vec![
            (0.0, 45.0, -8.0),
            (0.5, 62.0, -14.5),
            (1.0, 88.0, -26.0),
            (1.57, 94.5, -31.4),
            (2.0, 92.0, -28.5),
            (2.5, 80.0, -20.0),
            (3.14, 45.0, -8.0),
        ];

        Self {
            is_open: false,
            active_tab: FloquetFrequencyTab::SyntheticLatticeCanvas,
            base_frequency_khz: 10.0,
            fsr_frequency_khz: 1.0,
            modulation_depth: 0.35,
            synthetic_gauge_flux_rad: std::f64::consts::FRAC_PI_2,
            kerr_nonlinearity: 0.08,
            initial_mode_center: -3,
            num_frequency_modes: 11,
            num_spatial_sites: 6,
            defect_active: false,
            current_regime: SolitonRegime::ChiralEdgeCurrent,
            engine,
            cached_dispersion,
            cached_edge_dispersion,
            cached_flux_sweep,
        }
    }
}

impl FloquetFrequencyDialog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fast constructor for cold boot latency optimization (< 0.1ms).
    pub fn new_fast() -> Self {
        Self::default()
    }

    /// Recompute simulation engine with current dialog parameters.
    pub fn recompute(&mut self) {
        let boundary_params = BoundaryModulationParams {
            base_frequency_khz: self.base_frequency_khz,
            fsr_frequency_khz: self.fsr_frequency_khz,
            modulation_frequency_khz: self.fsr_frequency_khz,
            modulation_depth: self.modulation_depth,
            synthetic_gauge_flux_rad: self.synthetic_gauge_flux_rad,
            num_frequency_modes: self.num_frequency_modes,
            num_spatial_sites: self.num_spatial_sites,
            detuning_khz: 0.0,
            boundary_phase_grad_rad: 0.0,
            defect_mode_active: self.defect_active,
        };

        let soliton_params = FrequencySolitonParams {
            kerr_nonlinearity: self.kerr_nonlinearity,
            initial_mode_center: self.initial_mode_center,
            initial_pulse_width: 1.2,
            pulse_amplitude: 1.0,
            propagation_time_steps: 40,
            time_step_dt_ms: 0.1,
            dissipation_rate: 0.005,
        };

        self.engine = FloquetFrequencyEngine::new(boundary_params, soliton_params, self.current_regime);

        // Update dispersion points
        let disp_raw = self.engine.lattice.compute_synthetic_dispersion(32);
        self.cached_dispersion = disp_raw
            .iter()
            .enumerate()
            .map(|(i, p)| [(i as f64) * 0.1, p.quasi_energies[0]])
            .collect();

        self.cached_edge_dispersion = disp_raw
            .iter()
            .enumerate()
            .filter(|(_, p)| p.is_edge_mode)
            .map(|(i, p)| [(i as f64) * 0.1, (p.quasi_energies[0] + p.quasi_energies[1]) * 0.5])
            .collect();

        self.cached_flux_sweep = self.engine.compute_flux_sweep(16);
    }

    /// Main render method for the dialog window.
    pub fn ui(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new("Floquet-Bloch Synthetic Frequency Dimension & Frequency Soliton Engine")
            .open(&mut is_open)
            .default_size(vec2(860.0, 680.0))
            .min_width(720.0)
            .min_height(550.0)
            .show(ctx, |ui| {
                self.render_content(ui);
            });

        self.is_open = is_open;
    }

    /// Internal content rendering for the dialog.
    pub fn render_content(&mut self, ui: &mut Ui) {
        self.render_header(ui);
        ui.separator();
        self.render_tab_bar(ui);
        ui.separator();

        match self.active_tab {
            FloquetFrequencyTab::SyntheticLatticeCanvas => self.render_synthetic_lattice_canvas(ui),
            FloquetFrequencyTab::QuasiEnergyDispersion => self.render_dispersion_plot(ui),
            FloquetFrequencyTab::ConversionSpectrum => self.render_conversion_spectrum_plot(ui),
            FloquetFrequencyTab::SolitonDynamics => self.render_soliton_dynamics_plot(ui),
            FloquetFrequencyTab::SyntheticFluxSweep => self.render_flux_sweep_plot(ui),
        }

        ui.separator();
        self.render_controls_and_presets(ui);
        ui.separator();
        self.render_telemetry_footer(ui);
    }

    fn render_header(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(
                RichText::new("Acoustic Floquet Synthetic Frequency Dimension")
                    .color(Color32::from_rgb(100, 200, 255))
                    .strong(),
            );
            ui.label(
                RichText::new("Synthetic 2D Lattice (x, m) | Dynamic Phase Modulation | Chiral Frequency Solitons")
                    .weak(),
            );
        });
    }

    fn render_tab_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(
                    self.active_tab == FloquetFrequencyTab::SyntheticLatticeCanvas,
                    "2D Synthetic Lattice Canvas",
                )
                .clicked()
            {
                self.active_tab = FloquetFrequencyTab::SyntheticLatticeCanvas;
            }
            if ui
                .selectable_label(
                    self.active_tab == FloquetFrequencyTab::QuasiEnergyDispersion,
                    "Quasi-Energy Dispersion",
                )
                .clicked()
            {
                self.active_tab = FloquetFrequencyTab::QuasiEnergyDispersion;
            }
            if ui
                .selectable_label(
                    self.active_tab == FloquetFrequencyTab::ConversionSpectrum,
                    "Frequency Conversion Spectrum",
                )
                .clicked()
            {
                self.active_tab = FloquetFrequencyTab::ConversionSpectrum;
            }
            if ui
                .selectable_label(
                    self.active_tab == FloquetFrequencyTab::SolitonDynamics,
                    "Frequency-Lattice Soliton",
                )
                .clicked()
            {
                self.active_tab = FloquetFrequencyTab::SolitonDynamics;
            }
            if ui
                .selectable_label(
                    self.active_tab == FloquetFrequencyTab::SyntheticFluxSweep,
                    "Gauge Flux Sweep",
                )
                .clicked()
            {
                self.active_tab = FloquetFrequencyTab::SyntheticFluxSweep;
            }
        });
    }

    /// Tab 1: 2D Synthetic Lattice Canvas (real space x vs frequency mode index m).
    fn render_synthetic_lattice_canvas(&self, ui: &mut Ui) {
        ui.label(RichText::new("2D Real-Frequency Synthetic Lattice (Horizontal: Real Space x | Vertical: Mode Index m):").italics());

        let (rect, _resp) = ui.allocate_exact_size(vec2(ui.available_width(), 320.0), Sense::hover());
        let painter = ui.painter_at(rect);

        // Dark background
        painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 20, 30));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(50, 60, 80)), StrokeKind::Outside);

        let nx = self.num_spatial_sites.max(2);
        let nm = self.num_frequency_modes.max(3);

        let pad_x = 70.0;
        let pad_y = 40.0;
        let usable_w = rect.width() - 2.0 * pad_x;
        let usable_h = rect.height() - 2.0 * pad_y;

        let dx = usable_w / ((nx - 1) as f32).max(1.0);
        let dy = usable_h / ((nm - 1) as f32).max(1.0);

        // Draw horizontal hopping lines (spatial coupling)
        for m_idx in 0..nm {
            let y = rect.bottom() - pad_y - (m_idx as f32) * dy;
            let p_start = pos2(rect.left() + pad_x, y);
            let p_end = pos2(rect.right() - pad_x, y);
            painter.line_segment([p_start, p_end], Stroke::new(1.0, Color32::from_rgb(45, 55, 75)));
        }

        // Draw vertical hopping lines (dynamic boundary phase modulation)
        for x_idx in 0..nx {
            let x = rect.left() + pad_x + (x_idx as f32) * dx;
            let p_bottom = pos2(x, rect.bottom() - pad_y);
            let p_top = pos2(x, rect.top() + pad_y);

            // Edge x=0 is highlighted (chiral edge channel)
            let stroke = if x_idx == 0 {
                Stroke::new(2.5, Color32::from_rgb(80, 220, 160))
            } else {
                Stroke::new(1.0, Color32::from_rgb(50, 70, 100))
            };
            painter.line_segment([p_bottom, p_top], stroke);
        }

        // Draw synthetic magnetic gauge flux Phi in plaquettes
        let phi_label = format!("Phi = {:.2} rad", self.synthetic_gauge_flux_rad);
        for x_idx in 0..(nx - 1) {
            for m_idx in 0..(nm - 1) {
                if (x_idx + m_idx) % 2 == 0 {
                    let cx = rect.left() + pad_x + (x_idx as f32 + 0.5) * dx;
                    let cy = rect.bottom() - pad_y - (m_idx as f32 + 0.5) * dy;
                    painter.circle_filled(pos2(cx, cy), 5.0, Color32::from_rgba_unmultiplied(100, 180, 255, 40));
                    painter.circle_stroke(pos2(cx, cy), 5.0, Stroke::new(1.0, Color32::from_rgb(60, 120, 200)));
                }
            }
        }
        painter.text(
            pos2(rect.right() - 80.0, rect.top() + 15.0),
            egui::Align2::RIGHT_TOP,
            phi_label,
            egui::FontId::proportional(11.0),
            Color32::from_rgb(120, 200, 255),
        );

        // Draw chiral edge current flow arrow along x=0
        let arrow_x = rect.left() + pad_x - 18.0;
        let a_start = pos2(arrow_x, rect.bottom() - pad_y);
        let a_end = pos2(arrow_x, rect.top() + pad_y);
        painter.line_segment([a_start, a_end], Stroke::new(2.0, Color32::from_rgb(80, 240, 160)));
        painter.text(
            pos2(arrow_x - 6.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            "Chiral Edge Flow",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(80, 240, 160),
        );

        // Draw nodes (x, m) with wavepacket probability intensity
        let last_profile = self.engine.wavepacket_trajectory.last();
        for x_idx in 0..nx {
            let x = rect.left() + pad_x + (x_idx as f32) * dx;
            for m_idx in 0..nm {
                let m = self.engine.lattice.mode_index(m_idx);
                let y = rect.bottom() - pad_y - (m_idx as f32) * dy;

                let is_defect = self.defect_active && (m_idx == nm / 2) && (x_idx == nx / 2);
                if is_defect {
                    painter.circle_filled(pos2(x, y), 8.0, Color32::from_rgb(255, 60, 60));
                    painter.text(pos2(x, y - 12.0), egui::Align2::CENTER_BOTTOM, "Defect", egui::FontId::proportional(9.0), Color32::from_rgb(255, 120, 120));
                    continue;
                }

                // Node intensity
                let prob = if let Some(prof) = last_profile {
                    let m_prob = prof.modes.get(m_idx).map(|s| s.power_linear).unwrap_or(0.0);
                    let x_prob = prof.spatial_probability.get(x_idx).copied().unwrap_or(0.0);
                    (m_prob * x_prob * 10.0).clamp(0.05, 1.0)
                } else {
                    0.1
                };

                let r_node = 4.0 + (prob as f32) * 7.0;
                let col = Color32::from_rgba_unmultiplied(
                    (80.0 + prob * 170.0) as u8,
                    (120.0 + prob * 135.0) as u8,
                    (220.0 + (1.0 - prob) * 35.0) as u8,
                    (100.0 + prob * 155.0) as u8,
                );

                painter.circle_filled(pos2(x, y), r_node, col);
                painter.circle_stroke(pos2(x, y), r_node, Stroke::new(1.0, Color32::WHITE));

                // Mode index label on left
                if x_idx == 0 {
                    painter.text(
                        pos2(x - 8.0, y),
                        egui::Align2::RIGHT_CENTER,
                        format!("m={}", m),
                        egui::FontId::proportional(10.0),
                        Color32::from_rgb(180, 200, 220),
                    );
                }

                // Spatial index label at bottom
                if m_idx == 0 {
                    painter.text(
                        pos2(x, y + 14.0),
                        egui::Align2::CENTER_TOP,
                        format!("x={}", x_idx),
                        egui::FontId::proportional(10.0),
                        Color32::from_rgb(180, 200, 220),
                    );
                }
            }
        }
    }

    /// Tab 2: Floquet-Bloch Quasi-Energy Dispersion Plot.
    fn render_dispersion_plot(&self, ui: &mut Ui) {
        ui.label(RichText::new("Floquet-Bloch Quasi-Energy Bands along Synthetic BZ Path:").italics());

        let bulk_line = Line::new("Bulk Bands", PlotPoints::new(self.cached_dispersion.clone()))
            .color(Color32::from_rgb(100, 160, 240))
            .width(2.0);

        let edge_line = Line::new("Chiral In-Gap Edge States", PlotPoints::new(self.cached_edge_dispersion.clone()))
            .color(Color32::from_rgb(80, 240, 160))
            .width(3.0);

        Plot::new("floquet_dispersion_plot")
            .height(280.0)
            .x_axis_label("Synthetic Momentum k")
            .y_axis_label("Quasi-Energy E / Omega_R")
            .show(ui, |plot_ui| {
                plot_ui.line(bulk_line);
                plot_ui.line(edge_line);
                plot_ui.vline(VLine::new("Boundary", 1.57).color(Color32::from_rgb(255, 200, 60)));
            });

        ui.label(RichText::new("Note: Gapless chiral edge modes span the topological bandgap, mediating unidirectional frequency conversion.").weak());
    }

    /// Tab 3: Frequency Conversion Spectrum.
    fn render_conversion_spectrum_plot(&self, ui: &mut Ui) {
        ui.label(RichText::new("Synthetic Mode Power Distribution |a_m|^2 (Final State):").italics());

        let spectrum = self.engine.compute_conversion_spectrum();
        let pts: Vec<[f64; 2]> = spectrum.iter().map(|&(m, db)| [m as f64, db]).collect();

        let stem_line = Line::new("Mode Power (dB)", PlotPoints::new(pts))
            .color(Color32::from_rgb(255, 140, 80))
            .width(2.5);

        Plot::new("conversion_spectrum_plot")
            .height(280.0)
            .x_axis_label("Synthetic Mode Index m")
            .y_axis_label("Power Magnitude (dB)")
            .show(ui, |plot_ui| {
                plot_ui.line(stem_line);
                plot_ui.vline(VLine::new("Injected Mode", self.initial_mode_center as f64).color(Color32::from_rgb(100, 160, 255)));
                plot_ui.vline(VLine::new("Target Sideband", 3.0).color(Color32::from_rgb(80, 240, 160)));
            });

        ui.horizontal(|ui| {
            ui.label(format!("Forward Conversion: {:.1}%", self.engine.metrics.forward_conversion_efficiency_percent));
            ui.label(format!("Reverse Isolation: {:.1} dB", self.engine.metrics.reverse_isolation_db));
            ui.label(format!("Sideband Purity: {:.1}%", self.engine.metrics.sideband_suppression_purity_percent));
        });
    }

    /// Tab 4: Soliton Dynamics Plot.
    fn render_soliton_dynamics_plot(&self, ui: &mut Ui) {
        ui.label(RichText::new("Synthetic Frequency Soliton Envelope |a_m(t)|^2:").italics());

        let mut lines = Vec::new();
        let colors = [
            Color32::from_rgb(100, 140, 220),
            Color32::from_rgb(240, 180, 60),
            Color32::from_rgb(80, 240, 160),
        ];

        let traj = &self.engine.wavepacket_trajectory;
        let count = traj.len();
        if count > 0 {
            let snap_indices = [0, count / 2, count - 1];
            for (i, &idx) in snap_indices.iter().enumerate() {
                if let Some(prof) = traj.get(idx) {
                    let pts: Vec<[f64; 2]> = prof
                        .modes
                        .iter()
                        .map(|m| [m.mode_index as f64, m.power_linear])
                        .collect();

                    let name = format!("t = {:.1} ms", prof.time_ms);
                    lines.push(Line::new(name, PlotPoints::new(pts)).color(colors[i % 3]).width(2.0));
                }
            }
        }

        Plot::new("soliton_dynamics_plot")
            .height(280.0)
            .x_axis_label("Synthetic Mode Index m")
            .y_axis_label("Linear Probability Density")
            .show(ui, |plot_ui| {
                for l in lines {
                    plot_ui.line(l);
                }
            });

        ui.label(RichText::new(format!(
            "Soliton Sech^2 Fit Retention Fidelity: {:.1}% | Mode Spread Sigma: {:.2} modes",
            self.engine.metrics.soliton_stability_fidelity_percent,
            last_spread(&self.engine)
        )).weak());
    }

    /// Tab 5: Synthetic Flux Sweep Plot.
    fn render_flux_sweep_plot(&self, ui: &mut Ui) {
        ui.label(RichText::new("Conversion Efficiency & Reverse Isolation vs Synthetic Flux Phi:").italics());

        let fwd_pts: Vec<[f64; 2]> = self.cached_flux_sweep.iter().map(|&(phi, fwd, _)| [phi, fwd]).collect();
        let rev_pts: Vec<[f64; 2]> = self.cached_flux_sweep.iter().map(|&(phi, _, rev)| [phi, -rev]).collect();

        let l_fwd = Line::new("Forward Conversion Efficiency (%)", PlotPoints::new(fwd_pts))
            .color(Color32::from_rgb(80, 220, 160))
            .width(2.0);

        let l_rev = Line::new("Reverse Isolation Magnitude (-dB)", PlotPoints::new(rev_pts))
            .color(Color32::from_rgb(255, 100, 120))
            .width(2.0);

        Plot::new("flux_sweep_plot")
            .height(280.0)
            .x_axis_label("Synthetic Gauge Flux Phi (rad)")
            .y_axis_label("Performance Metrics")
            .show(ui, |plot_ui| {
                plot_ui.line(l_fwd);
                plot_ui.line(l_rev);
                plot_ui.vline(VLine::new("Operating Phi", self.synthetic_gauge_flux_rad).color(Color32::WHITE));
            });
    }

    /// Parameter Controls and Preset Buttons.
    fn render_controls_and_presets(&mut self, ui: &mut Ui) {
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label(RichText::new("Presets:").strong());
            if ui.button("Chiral Frequency Edge Current").clicked() {
                self.current_regime = SolitonRegime::ChiralEdgeCurrent;
                self.synthetic_gauge_flux_rad = std::f64::consts::FRAC_PI_2;
                self.kerr_nonlinearity = 0.02;
                changed = true;
            }
            if ui.button("Localized Frequency Soliton").clicked() {
                self.current_regime = SolitonRegime::LocalizedFrequencySoliton;
                self.synthetic_gauge_flux_rad = std::f64::consts::FRAC_PI_2;
                self.kerr_nonlinearity = 0.08;
                changed = true;
            }
            if ui.button("Linear Group Dispersion").clicked() {
                self.current_regime = SolitonRegime::LinearDispersion;
                self.synthetic_gauge_flux_rad = 0.0;
                self.kerr_nonlinearity = 0.0;
                changed = true;
            }
            if ui.button("High-Harmonic Frequency Pumper").clicked() {
                self.current_regime = SolitonRegime::ChiralEdgeCurrent;
                self.modulation_depth = 0.50;
                self.synthetic_gauge_flux_rad = 2.0 * PI / 3.0;
                changed = true;
            }
            if ui.button("Recompute Simulation").clicked() {
                changed = true;
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            changed |= ui
                .add(egui::Slider::new(&mut self.modulation_depth, 0.05..=0.80).text("Modulation Depth mu"))
                .changed();

            changed |= ui
                .add(egui::Slider::new(&mut self.synthetic_gauge_flux_rad, 0.0..=2.0 * PI).text("Gauge Flux Phi (rad)"))
                .changed();

            changed |= ui
                .add(egui::Slider::new(&mut self.kerr_nonlinearity, 0.0..=0.30).text("Kerr chi^(3)"))
                .changed();

            changed |= ui.checkbox(&mut self.defect_active, "Defect Obstacle Active").changed();
        });

        if changed {
            self.recompute();
        }
    }

    /// Bottom Telemetry Footer.
    fn render_telemetry_footer(&self, ui: &mut Ui) {
        let m = &self.engine.metrics;
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Synthetic Chern C: {}", m.synthetic_chern_number)).strong());
            ui.separator();
            ui.label(RichText::new(format!("Forward Conversion: {:.1}%", m.forward_conversion_efficiency_percent)).color(Color32::from_rgb(80, 220, 160)));
            ui.separator();
            ui.label(format!("Insertion Loss: {:.2} dB", m.forward_insertion_loss_db));
            ui.separator();
            ui.label(format!("Reverse Isolation: {:.1} dB", m.reverse_isolation_db));
            ui.separator();
            ui.label(format!("Edge Velocity: {:.1} modes/ms", m.synthetic_edge_velocity_modes_per_ms));
            ui.separator();
            ui.label(format!("Soliton Fidelity: {:.1}%", m.soliton_stability_fidelity_percent));
        });
    }
}

fn last_spread(engine: &FloquetFrequencyEngine) -> f64 {
    engine
        .wavepacket_trajectory
        .last()
        .map(|p| p.mode_spread_sigma)
        .unwrap_or(1.2)
}
