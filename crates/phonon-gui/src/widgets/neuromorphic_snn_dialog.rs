#![deny(unsafe_code)]

//! Interactive Neuromorphic Studio Canvas & Synaptic Weight Visualizer modal dialog.
//!
//! Provides real-time memristor crossbar synaptic weight colormap heatmap (Turbo/Magma),
//! LIF membrane potential traces with threshold reference lines, multi-channel spike
//! raster activity train, STDP parameter tuning, and physical energy telemetry (fJ/SOP).

use egui::{
    pos2, vec2, Color32, FontId, Rect, RichText, Sense, Stroke, StrokeKind, Ui,
};
use egui_plot::{Legend, Line, LineStyle, Plot, PlotPoints, Points};
use phonon_solver::neuromorphic::{
    SnnTrajectory, SpikingCrossbarNetwork, StdpParams, WindowFunction,
};

use crate::thermal::heatmap::{sample_colormap, Colormap};

/// Window function selector option for GUI combo box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiWindowFunction {
    Linear,
    Joglekar,
    Biolek,
}

/// Interactive modal dialog for neuromorphic SNN hardware co-design.
#[derive(Debug, Clone, PartialEq)]
pub struct NeuromorphicSnnDialog {
    /// Window visibility toggle.
    pub is_open: bool,

    // Crossbar dimensions
    pub num_inputs: usize,
    pub num_outputs: usize,

    // STDP learning parameters
    pub stdp_params: StdpParams,

    // Window function configuration
    pub window_func_type: GuiWindowFunction,
    pub window_p: f64,

    // Colormap selection
    pub colormap: Colormap,

    // SNN simulation network & cached trajectory
    pub network: SpikingCrossbarNetwork,
    pub trajectory: Option<SnnTrajectory>,

    // Simulation configuration
    pub duration_s: f64,
    pub dt: f64,
    pub pulse_voltage: f64,
    pub pulse_duration: f64,

    // Status and telemetry
    pub status_msg: String,
    pub run_requested: bool,
    pub hovered_synapse: Option<(usize, usize)>,
}

impl Default for NeuromorphicSnnDialog {
    fn default() -> Self {
        let mut dialog = Self {
            is_open: false,
            num_inputs: 8,
            num_outputs: 4,
            stdp_params: StdpParams::default(),
            window_func_type: GuiWindowFunction::Biolek,
            window_p: 2.0,
            colormap: Colormap::Turbo,
            network: SpikingCrossbarNetwork::new(8, 4),
            trajectory: None,
            duration_s: 0.10,      // 100 ms
            dt: 0.0005,            // 0.5 ms
            pulse_voltage: 1.5,    // 1.5 V
            pulse_duration: 0.001, // 1.0 ms
            status_msg: "Neuromorphic SNN Studio initialized.".to_string(),
            run_requested: false,
            hovered_synapse: None,
        };
        dialog.rebuild_network_and_run(true);
        dialog
    }
}

impl NeuromorphicSnnDialog {
    /// Creates a new dialog instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Reconstructs the internal crossbar network with current dimensions and parameters.
    pub fn rebuild_network_and_run(&mut self, apply_pattern: bool) {
        let window_func = match self.window_func_type {
            GuiWindowFunction::Linear => WindowFunction::Linear,
            GuiWindowFunction::Joglekar => WindowFunction::Joglekar { p: self.window_p },
            GuiWindowFunction::Biolek => WindowFunction::Biolek { p: self.window_p },
        };

        let mut net = SpikingCrossbarNetwork::new(self.num_inputs, self.num_outputs)
            .with_stdp_params(self.stdp_params)
            .with_pulse_voltage(self.pulse_voltage)
            .with_pulse_duration(self.pulse_duration);
        net.set_window_function(window_func);

        let input_trains = if apply_pattern {
            net.generate_pattern_stimulus(self.duration_s)
        } else {
            let mut trains = vec![Vec::new(); self.num_inputs];
            let mut t = 0.010;
            while t < self.duration_s {
                trains[0].push(t);
                if self.num_inputs > 1 {
                    trains[1].push(t + 0.002);
                }
                t += 0.020;
            }
            trains
        };

        let traj = net.simulate(self.duration_s, self.dt, &input_trains);
        self.trajectory = Some(traj);
        self.network = net;
        self.status_msg = format!(
            "Simulated {}x{} crossbar over {:.1} ms ({} steps).",
            self.num_inputs,
            self.num_outputs,
            self.duration_s * 1000.0,
            (self.duration_s / self.dt).round() as usize
        );
    }

    /// Runs simulation using current network weights and pattern stimulus.
    pub fn run_simulation(&mut self) {
        let input_trains = self.network.generate_pattern_stimulus(self.duration_s);
        let traj = self.network.simulate(self.duration_s, self.dt, &input_trains);
        self.trajectory = Some(traj);
        self.status_msg = "SNN simulation completed successfully.".to_string();
    }

    /// Applies custom pattern stimulus to excite specific crossbar pathways.
    pub fn apply_pattern_stimulus(&mut self) {
        let window_func = match self.window_func_type {
            GuiWindowFunction::Linear => WindowFunction::Linear,
            GuiWindowFunction::Joglekar => WindowFunction::Joglekar { p: self.window_p },
            GuiWindowFunction::Biolek => WindowFunction::Biolek { p: self.window_p },
        };
        self.network.set_window_function(window_func);
        self.network.stdp_params = self.stdp_params;
        self.network.pulse_voltage = self.pulse_voltage;
        self.network.pulse_duration = self.pulse_duration;

        let input_trains = self.network.generate_pattern_stimulus(self.duration_s);
        let traj = self.network.simulate(self.duration_s, self.dt, &input_trains);
        self.trajectory = Some(traj);
        self.status_msg = "Pattern stimulus applied with online STDP weight updates.".to_string();
    }

    /// Resets synaptic weights to a uniform baseline.
    pub fn reset_weights(&mut self) {
        self.network.reset_weights(0.2);
        self.run_simulation();
        self.status_msg = "Synaptic weights reset to baseline w = 0.20.".to_string();
    }

    /// Modal window display entry point.
    pub fn show(&mut self, ctx: &egui::Context) {
        self.ui(ctx);
    }

    /// Renders modal window.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Neuromorphic SNN Studio - Synaptic Crossbar & STDP Engine")
            .open(&mut is_open)
            .default_size([960.0, 720.0])
            .min_size([750.0, 540.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders dialog internal content.
    pub fn render_content(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // 1. Top Control Panel
            self.render_control_panel(ui);
            ui.add_space(6.0);

            // 2. Physical Telemetry Cards
            self.render_telemetry_panel(ui);
            ui.add_space(8.0);

            // 3. Middle Section: Synaptic Colormap Heatmap
            ui.horizontal(|ui| {
                ui.heading(
                    RichText::new("Synaptic Crossbar Conductance Heatmap (M x N)")
                        .size(13.0)
                        .color(Color32::from_rgb(180, 205, 235)),
                );
                ui.label(
                    RichText::new(format!(
                        "| Size: {} In x {} Out | Colormap: {:?}",
                        self.num_inputs, self.num_outputs, self.colormap
                    ))
                    .size(11.0)
                    .color(Color32::from_rgb(120, 140, 165)),
                );
            });
            ui.add_space(3.0);
            self.render_heatmap(ui);
            ui.add_space(8.0);

            // 4. Membrane Potential Traces
            ui.heading(
                RichText::new("LIF Neuron Membrane Potential Traces V_j(t)")
                    .size(13.0)
                    .color(Color32::from_rgb(180, 205, 235)),
            );
            ui.add_space(3.0);
            self.render_membrane_traces(ui);
            ui.add_space(8.0);

            // 5. Spike Raster Activity Train
            ui.heading(
                RichText::new("Spike Raster Activity Train (Pre-Synaptic & Post-Synaptic)")
                    .size(13.0)
                    .color(Color32::from_rgb(180, 205, 235)),
            );
            ui.add_space(3.0);
            self.render_spike_raster(ui);
        });
    }

    /// Renders control panel with network sizing, STDP tuning, and action buttons.
    fn render_control_panel(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(8.0, 4.0);

                // Network sizing
                ui.label(RichText::new("Inputs M:").size(11.0).color(Color32::from_rgb(180, 200, 220)));
                let m_changed = ui
                    .add(egui::DragValue::new(&mut self.num_inputs).range(1..=32).speed(1))
                    .changed();

                ui.label(RichText::new("Outputs N:").size(11.0).color(Color32::from_rgb(180, 200, 220)));
                let n_changed = ui
                    .add(egui::DragValue::new(&mut self.num_outputs).range(1..=16).speed(1))
                    .changed();

                // Quick presets
                if ui.button("8x4").clicked() {
                    self.num_inputs = 8;
                    self.num_outputs = 4;
                    self.rebuild_network_and_run(true);
                }
                if ui.button("16x8").clicked() {
                    self.num_inputs = 16;
                    self.num_outputs = 8;
                    self.rebuild_network_and_run(true);
                }

                ui.separator();

                // Window Function
                ui.label(RichText::new("Window:").size(11.0).color(Color32::from_rgb(180, 200, 220)));
                let w_str = match self.window_func_type {
                    GuiWindowFunction::Linear => "Linear",
                    GuiWindowFunction::Joglekar => "Joglekar",
                    GuiWindowFunction::Biolek => "Biolek",
                };
                egui::ComboBox::from_id_salt("snn_window_func_combo")
                    .selected_text(w_str)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.window_func_type, GuiWindowFunction::Biolek, "Biolek");
                        ui.selectable_value(&mut self.window_func_type, GuiWindowFunction::Joglekar, "Joglekar");
                        ui.selectable_value(&mut self.window_func_type, GuiWindowFunction::Linear, "Linear");
                    });

                // Colormap
                ui.label(RichText::new("Colormap:").size(11.0).color(Color32::from_rgb(180, 200, 220)));
                let cmap_str = match self.colormap {
                    Colormap::Turbo => "Turbo",
                    Colormap::Magma => "Magma",
                    Colormap::Inferno => "Inferno",
                };
                egui::ComboBox::from_id_salt("snn_colormap_combo")
                    .selected_text(cmap_str)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.colormap, Colormap::Turbo, "Turbo");
                        ui.selectable_value(&mut self.colormap, Colormap::Magma, "Magma");
                        ui.selectable_value(&mut self.colormap, Colormap::Inferno, "Inferno");
                    });

                if m_changed || n_changed {
                    self.rebuild_network_and_run(true);
                }
            });

            ui.separator();

            // STDP parameters row
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(8.0, 4.0);

                ui.label(RichText::new("A_+ (LTP):").size(11.0).color(Color32::from_rgb(180, 200, 220)));
                ui.add(egui::DragValue::new(&mut self.stdp_params.a_plus).range(0.001..=0.5).speed(0.005));

                ui.label(RichText::new("A_- (LTD):").size(11.0).color(Color32::from_rgb(180, 200, 220)));
                ui.add(egui::DragValue::new(&mut self.stdp_params.a_minus).range(0.001..=0.5).speed(0.005));

                let mut tau_p_ms = self.stdp_params.tau_plus * 1000.0;
                ui.label(RichText::new("tau_+:").size(11.0).color(Color32::from_rgb(180, 200, 220)));
                if ui
                    .add(egui::DragValue::new(&mut tau_p_ms).range(1.0..=100.0).speed(0.5).suffix(" ms"))
                    .changed()
                {
                    self.stdp_params.tau_plus = tau_p_ms / 1000.0;
                }

                let mut tau_m_ms = self.stdp_params.tau_minus * 1000.0;
                ui.label(RichText::new("tau_-:").size(11.0).color(Color32::from_rgb(180, 200, 220)));
                if ui
                    .add(egui::DragValue::new(&mut tau_m_ms).range(1.0..=100.0).speed(0.5).suffix(" ms"))
                    .changed()
                {
                    self.stdp_params.tau_minus = tau_m_ms / 1000.0;
                }

                ui.label(RichText::new("V_pulse:").size(11.0).color(Color32::from_rgb(180, 200, 220)));
                ui.add(
                    egui::DragValue::new(&mut self.pulse_voltage)
                        .range(0.1..=5.0)
                        .speed(0.1)
                        .suffix(" V"),
                );
            });

            ui.separator();

            // Action buttons row
            ui.horizontal(|ui| {
                if ui
                    .button(RichText::new("Run SNN Simulation").strong().color(Color32::from_rgb(56, 189, 248)))
                    .clicked()
                {
                    self.run_simulation();
                }

                if ui
                    .button(RichText::new("Apply Pattern Stimulus").strong().color(Color32::from_rgb(34, 197, 94)))
                    .clicked()
                {
                    self.apply_pattern_stimulus();
                }

                if ui
                    .button(RichText::new("Reset Weights").color(Color32::from_rgb(239, 68, 68)))
                    .clicked()
                {
                    self.reset_weights();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(&self.status_msg)
                            .size(11.0)
                            .color(Color32::from_rgb(160, 180, 200)),
                    );
                });
            });
        });
    }

    /// Renders physical telemetry cards.
    fn render_telemetry_panel(&self, ui: &mut Ui) {
        let (total_spikes, mean_g_us, energy_fj_sop, avg_freq_hz) = if let Some(ref traj) = self.trajectory {
            let total = traj.input_spikes.len() + traj.output_spikes.len();
            let mean_g = traj.mean_conductance_s * 1.0e6;
            let avg_freq = if !traj.output_spike_rates_hz.is_empty() {
                traj.output_spike_rates_hz.iter().sum::<f64>() / traj.output_spike_rates_hz.len() as f64
            } else {
                0.0
            };
            (total, mean_g, traj.energy_per_sop_fj, avg_freq)
        } else {
            (0, 0.0, 0.0, 0.0)
        };

        ui.horizontal(|ui| {
            ui.columns(4, |cols| {
                // Card 1: Total Spikes
                cols[0].group(|ui| {
                    ui.label(RichText::new("Total Spikes").size(10.5).color(Color32::from_rgb(140, 160, 185)));
                    ui.label(RichText::new(format!("{}", total_spikes)).size(16.0).strong().color(Color32::from_rgb(56, 189, 248)));
                });

                // Card 2: Mean Conductance
                cols[1].group(|ui| {
                    ui.label(RichText::new("Mean Conductance").size(10.5).color(Color32::from_rgb(140, 160, 185)));
                    ui.label(RichText::new(format!("{:.1} uS", mean_g_us)).size(16.0).strong().color(Color32::from_rgb(52, 211, 153)));
                });

                // Card 3: Energy per Operation
                cols[2].group(|ui| {
                    ui.label(RichText::new("Synaptic Energy").size(10.5).color(Color32::from_rgb(140, 160, 185)));
                    ui.label(RichText::new(format!("{:.2} fJ/SOP", energy_fj_sop)).size(16.0).strong().color(Color32::from_rgb(251, 191, 36)));
                });

                // Card 4: Average Firing Frequency
                cols[3].group(|ui| {
                    ui.label(RichText::new("Avg Firing Freq").size(10.5).color(Color32::from_rgb(140, 160, 185)));
                    ui.label(RichText::new(format!("{:.1} Hz", avg_freq_hz)).size(16.0).strong().color(Color32::from_rgb(244, 114, 182)));
                });
            });
        });
    }

    /// Renders M x N crossbar synaptic weight colormap heatmap.
    fn render_heatmap(&mut self, ui: &mut Ui) {
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width().max(240.0), 200.0), Sense::hover());
        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 20, 28));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(35, 45, 60)), StrokeKind::Outside);

        let m = self.num_inputs.max(1);
        let n = self.num_outputs.max(1);

        let label_left = 40.0;
        let label_top = 22.0;
        let grid_rect = Rect::from_min_max(
            pos2(rect.min.x + label_left, rect.min.y + label_top),
            pos2(rect.max.x - 10.0, rect.max.y - 24.0),
        );

        let cell_w = grid_rect.width() / n as f32;
        let cell_h = grid_rect.height() / m as f32;

        let mouse_pos = response.hover_pos();
        let mut hovered = None;

        // Draw column headers (Outputs)
        for j in 0..n {
            let cx = grid_rect.min.x + (j as f32 + 0.5) * cell_w;
            painter.text(
                pos2(cx, rect.min.y + 11.0),
                egui::Align2::CENTER_CENTER,
                format!("Out{}", j),
                FontId::proportional(10.0),
                Color32::from_rgb(130, 150, 175),
            );
        }

        // Draw row headers (Inputs)
        for i in 0..m {
            let cy = grid_rect.min.y + (i as f32 + 0.5) * cell_h;
            painter.text(
                pos2(rect.min.x + 20.0, cy),
                egui::Align2::CENTER_CENTER,
                format!("In{}", i),
                FontId::proportional(10.0),
                Color32::from_rgb(130, 150, 175),
            );
        }

        // Draw cells
        let conductances = if let Some(ref traj) = self.trajectory {
            traj.final_conductances.clone()
        } else {
            self.network.conductance_matrix()
        };

        let weights = self.network.weight_matrix();

        for i in 0..m {
            for j in 0..n {
                let cell_min = pos2(grid_rect.min.x + j as f32 * cell_w, grid_rect.min.y + i as f32 * cell_h);
                let cell_max = pos2(cell_min.x + cell_w - 1.0, cell_min.y + cell_h - 1.0);
                let cell_rect = Rect::from_min_max(cell_min, cell_max);

                let _g_val = if i < conductances.len() && j < conductances[i].len() {
                    conductances[i][j]
                } else {
                    1e-5
                };

                // State variable w is closely tied to conductance
                let w_val = if i < weights.len() && j < weights[i].len() {
                    weights[i][j]
                } else {
                    0.2
                };

                let norm_u = w_val.clamp(0.0, 1.0) as f32;
                let color = sample_colormap(norm_u, self.colormap);

                painter.rect_filled(cell_rect, 1.0, color);

                if let Some(pos) = mouse_pos {
                    if cell_rect.contains(pos) {
                        hovered = Some((i, j));
                        painter.rect_stroke(
                            cell_rect,
                            1.0,
                            Stroke::new(2.0, Color32::from_rgb(255, 255, 255)),
                            StrokeKind::Inside,
                        );
                    }
                }
            }
        }

        // Color bar legend at bottom
        let bar_rect = Rect::from_min_max(
            pos2(grid_rect.min.x, rect.max.y - 16.0),
            pos2(grid_rect.max.x, rect.max.y - 6.0),
        );
        let bar_w = bar_rect.width();
        let segments = 32;
        for s in 0..segments {
            let u0 = s as f32 / segments as f32;
            let u1 = (s + 1) as f32 / segments as f32;
            let sub_rect = Rect::from_min_max(
                pos2(bar_rect.min.x + u0 * bar_w, bar_rect.min.y),
                pos2(bar_rect.min.x + u1 * bar_w, bar_rect.max.y),
            );
            painter.rect_filled(sub_rect, 0.0, sample_colormap(u0, self.colormap));
        }

        self.hovered_synapse = hovered;
        if let Some((hi, hj)) = hovered {
            let g = if hi < conductances.len() && hj < conductances[hi].len() {
                conductances[hi][hj]
            } else {
                0.0
            };
            let r_kohm = 1.0 / g.max(1e-12) / 1000.0;
            let w = if hi < weights.len() && hj < weights[hi].len() {
                weights[hi][hj]
            } else {
                0.0
            };
            response.show_tooltip_ui(|ui| {
                ui.label(RichText::new(format!("Synapse [In {} -> Out {}]", hi, hj)).strong());
                ui.label(format!("Conductance: {:.2} uS", g * 1.0e6));
                ui.label(format!("Resistance: {:.1} kOhm", r_kohm));
                ui.label(format!("State variable w: {:.3}", w));
            });
        }
    }

    /// Renders LIF neuron membrane potential traces over time.
    fn render_membrane_traces(&self, ui: &mut Ui) {
        let plot = Plot::new("snn_membrane_potential_traces")
            .legend(Legend::default())
            .x_axis_label("Time (s)")
            .y_axis_label("Membrane Potential V (V)")
            .height(160.0)
            .allow_zoom(true)
            .allow_drag(true);

        plot.show(ui, |plot_ui| {
            if let Some(ref traj) = self.trajectory {
                let colors = [
                    Color32::from_rgb(56, 189, 248),  // Light blue
                    Color32::from_rgb(251, 146, 60),  // Orange
                    Color32::from_rgb(167, 139, 250), // Purple
                    Color32::from_rgb(52, 211, 153),  // Emerald
                    Color32::from_rgb(244, 114, 182), // Pink
                    Color32::from_rgb(250, 204, 21),  // Amber
                ];

                for (j, trace) in traj.membrane_potentials.iter().enumerate() {
                    let pts: Vec<[f64; 2]> = traj
                        .timestamps
                        .iter()
                        .copied()
                        .zip(trace.iter().copied())
                        .map(|(t, v)| [t, v])
                        .collect();
                    let color = colors[j % colors.len()];
                    plot_ui.line(
                        Line::new(format!("Neuron {}", j), PlotPoints::from(pts))
                            .color(color)
                            .width(1.8),
                    );
                }

                // Dashed line at threshold V_th = 1.0 V
                if let Some(&t_max) = traj.timestamps.last() {
                    let v_th = self.network.neurons.first().map(|n| n.v_th).unwrap_or(1.0);
                    let th_pts = vec![[0.0, v_th], [t_max, v_th]];
                    plot_ui.line(
                        Line::new("Threshold V_th", PlotPoints::from(th_pts))
                            .color(Color32::from_rgb(239, 68, 68))
                            .style(LineStyle::Dashed { length: 8.0 })
                            .width(1.5),
                    );
                }
            }
        });
    }

    /// Renders spike raster activity train across inputs and outputs.
    fn render_spike_raster(&self, ui: &mut Ui) {
        let plot = Plot::new("snn_spike_raster_plot")
            .legend(Legend::default())
            .x_axis_label("Time (s)")
            .y_axis_label("Channel / Neuron Index")
            .height(150.0)
            .allow_zoom(true)
            .allow_drag(true);

        plot.show(ui, |plot_ui| {
            if let Some(ref traj) = self.trajectory {
                let mut in_pts: Vec<[f64; 2]> = Vec::with_capacity(traj.input_spikes.len());
                for &(t, ch) in &traj.input_spikes {
                    in_pts.push([t, ch as f64]);
                }

                let mut out_pts: Vec<[f64; 2]> = Vec::with_capacity(traj.output_spikes.len());
                let offset = self.num_inputs as f64 + 1.0;
                for &(t, neuron_idx) in &traj.output_spikes {
                    out_pts.push([t, offset + neuron_idx as f64]);
                }

                // Plot input spike points
                if !in_pts.is_empty() {
                    plot_ui.points(
                        Points::new("Input Spikes", PlotPoints::from(in_pts))
                            .color(Color32::from_rgb(56, 189, 248))
                            .radius(3.0),
                    );
                }

                // Plot output spike points
                if !out_pts.is_empty() {
                    plot_ui.points(
                        Points::new("Output Spikes", PlotPoints::from(out_pts))
                            .color(Color32::from_rgb(251, 146, 60))
                            .radius(3.5),
                    );
                }

                // Separator line between inputs and outputs
                if let Some(&t_max) = traj.timestamps.last() {
                    let sep_y = self.num_inputs as f64 + 0.5;
                    plot_ui.line(
                        Line::new("Boundary", PlotPoints::from(vec![[0.0, sep_y], [t_max, sep_y]]))
                            .color(Color32::from_rgb(80, 95, 115))
                            .style(LineStyle::Dotted { spacing: 6.0 })
                            .width(1.0),
                    );
                }
            }
        });
    }
}
