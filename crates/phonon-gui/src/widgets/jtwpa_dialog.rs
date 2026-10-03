#![deny(unsafe_code)]

//! Interactive Josephson Traveling-Wave Parametric Amplifier (JTWPA) Studio modal dialog.
//!
//! Provides:
//! - Discrete Transmission Line Lattice Canvas: 2D schematic diagram of the N-cell Josephson LC ladder,
//!   rendering series Josephson junctions (cross symbol X), ground capacitors, and periodic RPM resonant shunt stubs.
//! - Continuous Gain Spectrum Plot: egui_plot rendering G(f) in dB across 4.0 - 8.0 GHz octave band
//!   with 20 dB gain threshold dashed line and 3-dB bandwidth markers.
//! - Phase Matching Dispersion Plot: plots Delta_k(f) with and without RPM engineering, showing how RPM
//!   compensates for nonlinear Kerr phase shift to achieve phase matching Delta_k approx 0.
//! - Quantum Quadrature Squeezing Ellipse: visualizes the squeezed state uncertainty ellipse in phase space (X1, X2)
//!   against the circular Standard Quantum Limit (SQL circle), with major/minor axes and squeezing level in dB.
//! - Telemetry Footer: Peak Gain (dB), 3-dB Bandwidth (GHz), Squeezing (dB below SQL), Added Noise Photon (n_add),
//!   Pump Power (dBm), Phase Mismatch Delta_k (rad/m).

use egui::{
    pos2, vec2, Color32, FontId, Rect, RichText, Sense, Stroke, StrokeKind, Ui,
};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use phonon_solver::jtwpa_simulator::{
    GainSpectrumPoint, JosephsonCellParams, JosephsonTransmissionLine, JtwpaParams, JtwpaSolver,
    QuantumSqueezing, RpmStubParams,
};

/// Interactive modal dialog for Superconducting JTWPA simulation and CAD visualization.
#[derive(Debug, Clone, PartialEq)]
pub struct JtwpaDialog {
    /// Window visibility toggle.
    pub is_open: bool,

    // Physical Controls
    /// Pump microwave frequency f_p in GHz (default 6.0 GHz).
    pub pump_freq_ghz: f64,
    /// Microwave generator pump power in dBm (default -50.0 dBm).
    pub pump_power_dbm: f64,
    /// Total number of discrete Josephson cells N (default 1000 cells).
    pub num_cells: usize,
    /// Small-signal linear Josephson inductance L_J0 in pH (default 80.0 pH).
    pub l_j0_ph: f64,
    /// Junction critical current I_c in uA (default 5.0 uA).
    pub i_c_ua: f64,
    /// Ground shunt capacitance C_g in fF (default 50.0 fF).
    pub c_g_ff: f64,
    /// Junction intrinsic geometric capacitance C_J in fF (default 10.0 fF).
    pub c_j_ff: f64,
    /// Physical cell length a in um (default 10.0 um).
    pub a_um: f64,
    /// Whether Resonant Phase Matching (RPM) is enabled.
    pub rpm_enabled: bool,
    /// RPM stub insertion period M_cells (default 16 cells).
    pub m_cells: usize,
    /// RPM stopband resonant frequency in GHz (default 12.0 GHz).
    pub f_rpm_ghz: f64,
    /// Signal sweep start frequency in GHz (default 4.0 GHz).
    pub signal_start_ghz: f64,
    /// Signal sweep stop frequency in GHz (default 8.0 GHz).
    pub signal_stop_ghz: f64,
    /// Cryogenic line attenuation from room-temp synthesizer to on-chip device in dB (default 34.0 dB).
    pub attenuation_db: f64,

    // Cached Simulation State
    /// Discrete gain spectrum points.
    pub spectrum: Vec<GainSpectrumPoint>,
    /// Signal power gain curve points [f_s (GHz), G_s (dB)].
    pub gain_curve: Vec<[f64; 2]>,
    /// Idler power gain curve points [f_s (GHz), G_i (dB)].
    pub idler_gain_curve: Vec<[f64; 2]>,
    /// Phase mismatch curve without RPM [f_s (GHz), Delta_k (rad/m)].
    pub dispersion_bare_curve: Vec<[f64; 2]>,
    /// Phase mismatch curve with RPM [f_s (GHz), Delta_k_rpm (rad/m)].
    pub dispersion_rpm_curve: Vec<[f64; 2]>,
    /// Quantum squeezing performance at center frequency.
    pub squeezing: QuantumSqueezing,

    // Telemetry Metrics
    /// Maximum peak signal gain across the octave band in dB.
    pub peak_gain_db: f64,
    /// 3-dB gain amplification bandwidth in GHz.
    pub bandwidth_3db_ghz: f64,
    /// Squeezing level in dB below Standard Quantum Limit.
    pub squeezing_db: f64,
    /// Quantum-limited added noise quanta n_add.
    pub n_add: f64,
    /// Effective on-chip pump power in dBm.
    pub on_chip_power_dbm: f64,
    /// Total phase mismatch Delta_k in rad/m at center signal frequency.
    pub delta_k_rad_per_m: f64,

    /// Execution rerun flag.
    pub run_requested: bool,
}

impl Default for JtwpaDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl JtwpaDialog {
    /// Creates a new JTWPA dialog initialized with default physical parameters and solved state.
    pub fn new() -> Self {
        let mut dialog = Self {
            is_open: false,
            pump_freq_ghz: 6.0,
            pump_power_dbm: -50.0,
            num_cells: 1000,
            l_j0_ph: 80.0,
            i_c_ua: 5.0,
            c_g_ff: 50.0,
            c_j_ff: 10.0,
            a_um: 10.0,
            rpm_enabled: true,
            m_cells: 16,
            f_rpm_ghz: 12.0,
            signal_start_ghz: 4.0,
            signal_stop_ghz: 8.0,
            attenuation_db: 34.0,

            spectrum: Vec::new(),
            gain_curve: Vec::new(),
            idler_gain_curve: Vec::new(),
            dispersion_bare_curve: Vec::new(),
            dispersion_rpm_curve: Vec::new(),
            squeezing: QuantumSqueezing::from_gain(100.0),

            peak_gain_db: 20.0,
            bandwidth_3db_ghz: 4.0,
            squeezing_db: 26.0,
            n_add: 0.495,
            on_chip_power_dbm: -84.0,
            delta_k_rad_per_m: 0.0,

            run_requested: false,
        };
        dialog.recompute();
        dialog
    }

    /// Recomputes the JTWPA simulation curves, phase matching, and quantum squeezing metrics.
    pub fn recompute(&mut self) {
        let cell_params = JosephsonCellParams::new(
            self.l_j0_ph * 1.0e-12,
            self.i_c_ua * 1.0e-6,
            self.c_g_ff * 1.0e-15,
            self.c_j_ff * 1.0e-15,
            self.a_um * 1.0e-6,
        );

        let rpm_params = if self.rpm_enabled {
            Some(RpmStubParams::from_stopband_freq(
                self.m_cells,
                self.f_rpm_ghz * 1.0e9,
                self.l_j0_ph * 1.0e-12,
            ))
        } else {
            None
        };

        let line = JosephsonTransmissionLine::new(self.num_cells, cell_params, rpm_params);
        let params = JtwpaParams::new(
            line,
            self.pump_freq_ghz * 1.0e9,
            self.pump_power_dbm,
            self.signal_start_ghz * 1.0e9,
            self.signal_stop_ghz * 1.0e9,
            100,
            self.attenuation_db,
        );

        self.on_chip_power_dbm = params.on_chip_pump_power_dbm();
        let spectrum = JtwpaSolver::solve_spectrum(&params);

        let mut g_pts = Vec::with_capacity(spectrum.len());
        let mut gi_pts = Vec::with_capacity(spectrum.len());
        let mut dk_bare_pts = Vec::with_capacity(spectrum.len());
        let mut dk_rpm_pts = Vec::with_capacity(spectrum.len());

        for pt in &spectrum {
            g_pts.push([pt.f_s, pt.gain_s_db]);
            gi_pts.push([pt.f_s, pt.gain_i_db]);
            dk_bare_pts.push([pt.f_s, pt.delta_k]);
            dk_rpm_pts.push([pt.f_s, pt.delta_k_rpm]);
        }

        let (peak_g, bw) = JtwpaSolver::bandwidth_3db(&spectrum);
        self.peak_gain_db = peak_g;
        self.bandwidth_3db_ghz = bw;

        // Center frequency squeezing calculation
        let center_f = (self.signal_start_ghz + self.signal_stop_ghz) * 0.5 * 1.0e9;
        self.squeezing = JtwpaSolver::quantum_squeezing(&params, center_f);
        self.squeezing_db = self.squeezing.squeezing_db;
        self.n_add = self.squeezing.n_add;

        // Center frequency phase mismatch
        let center_point = JtwpaSolver::solve_point(&params, center_f, self.rpm_enabled);
        self.delta_k_rad_per_m = if self.rpm_enabled {
            center_point.delta_k_rpm
        } else {
            center_point.delta_k
        };

        self.spectrum = spectrum;
        self.gain_curve = g_pts;
        self.idler_gain_curve = gi_pts;
        self.dispersion_bare_curve = dk_bare_pts;
        self.dispersion_rpm_curve = dk_rpm_pts;
    }

    /// Renders the modal window into the given egui Context.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Phonon Studio Superconducting Josephson Traveling-Wave Parametric Amplifier Simulator")
            .open(&mut is_open)
            .default_size([1040.0, 720.0])
            .min_size([820.0, 580.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders controls bar, 2x2 visualizer canvases, and telemetry footer.
    pub fn render_content(&mut self, ui: &mut Ui) {
        let mut recompute = false;

        // 1. Top Controls Bar
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(10.0, 4.0);

            // Pump Power Slider
            ui.label(RichText::new("Pump Power:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui
                .add(
                    egui::Slider::new(&mut self.pump_power_dbm, -65.0..=-40.0)
                        .step_by(0.5)
                        .suffix(" dBm"),
                )
                .changed()
            {
                recompute = true;
            }

            ui.separator();

            // Pump Frequency Slider
            ui.label(RichText::new("Pump Freq:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui
                .add(
                    egui::Slider::new(&mut self.pump_freq_ghz, 5.0..=7.0)
                        .step_by(0.1)
                        .suffix(" GHz"),
                )
                .changed()
            {
                recompute = true;
            }

            ui.separator();

            // Cell Count Slider
            ui.label(RichText::new("Cells N:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui
                .add(
                    egui::Slider::new(&mut self.num_cells, 200..=4000)
                        .step_by(50.0),
                )
                .changed()
            {
                recompute = true;
            }

            ui.separator();

            // RPM Toggle
            if ui
                .checkbox(&mut self.rpm_enabled, RichText::new("RPM Engineering").size(11.0).color(Color32::from_rgb(100, 240, 210)))
                .changed()
            {
                recompute = true;
            }

            // RPM Period Slider
            if self.rpm_enabled {
                ui.label(RichText::new("M:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
                if ui
                    .add(egui::Slider::new(&mut self.m_cells, 8..=32).step_by(4.0))
                    .changed()
                {
                    recompute = true;
                }
            }

            ui.separator();

            // Inductance Slider
            ui.label(RichText::new("L_J0:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui
                .add(
                    egui::Slider::new(&mut self.l_j0_ph, 40.0..=150.0)
                        .step_by(5.0)
                        .suffix(" pH"),
                )
                .changed()
            {
                recompute = true;
            }

            // Critical Current Slider
            ui.label(RichText::new("I_c:").size(11.0).color(Color32::from_rgb(180, 205, 230)));
            if ui
                .add(
                    egui::Slider::new(&mut self.i_c_ua, 1.0..=10.0)
                        .step_by(0.5)
                        .suffix(" uA"),
                )
                .changed()
            {
                recompute = true;
            }

            // Reset Button
            if ui.button("Reset Defaults").clicked() {
                self.pump_freq_ghz = 6.0;
                self.pump_power_dbm = -50.0;
                self.num_cells = 1000;
                self.l_j0_ph = 80.0;
                self.i_c_ua = 5.0;
                self.c_g_ff = 50.0;
                self.c_j_ff = 10.0;
                self.a_um = 10.0;
                self.rpm_enabled = true;
                self.m_cells = 16;
                self.f_rpm_ghz = 12.0;
                self.attenuation_db = 34.0;
                recompute = true;
            }
        });

        if recompute || self.run_requested {
            self.run_requested = false;
            self.recompute();
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // 2. Main 2x2 Grid Visualization Canvas
        let total_avail = ui.available_size();
        let quad_w = ((total_avail.x - 12.0) * 0.5).max(380.0);
        let quad_h = ((total_avail.y - 120.0) * 0.5).max(220.0);

        ui.horizontal(|ui| {
            // Panel 1: Discrete Transmission Line Lattice Canvas
            ui.vertical(|ui| {
                ui.set_width(quad_w);
                ui.set_height(quad_h);
                ui.label(
                    RichText::new("Discrete Josephson LC Transmission Line Lattice")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(120, 200, 255)),
                );
                self.render_lattice_canvas(ui, quad_w, quad_h - 22.0);
            });

            ui.separator();

            // Panel 2: Continuous Gain Spectrum Plot
            ui.vertical(|ui| {
                ui.set_width(quad_w);
                ui.set_height(quad_h);
                ui.label(
                    RichText::new("Continuous 4WM Gain Spectrum G(f) across Octave Band")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(100, 240, 210)),
                );
                self.render_gain_spectrum_plot(ui, quad_w, quad_h - 22.0);
            });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            // Panel 3: Phase Matching Dispersion Plot
            ui.vertical(|ui| {
                ui.set_width(quad_w);
                ui.set_height(quad_h);
                ui.label(
                    RichText::new("Phase Matching Dispersion Delta_k(f) (With vs Without RPM)")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(255, 190, 80)),
                );
                self.render_dispersion_plot(ui, quad_w, quad_h - 22.0);
            });

            ui.separator();

            // Panel 4: Quantum Quadrature Squeezing Ellipse
            ui.vertical(|ui| {
                ui.set_width(quad_w);
                ui.set_height(quad_h);
                ui.label(
                    RichText::new("Quantum Quadrature Squeezing Phase Space Ellipse")
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(230, 140, 255)),
                );
                self.render_squeezing_canvas(ui, quad_w, quad_h - 22.0);
            });
        });

        ui.add_space(6.0);
        ui.separator();

        // 3. Telemetry Footer
        self.render_telemetry_footer(ui);
    }

    /// Renders 2D schematic diagram of the discrete Josephson LC ladder network.
    fn render_lattice_canvas(&self, ui: &mut Ui, width: f32, height: f32) {
        let (response, painter) = ui.allocate_painter(vec2(width, height), Sense::hover());
        let rect = response.rect;

        painter.rect_filled(rect, 4.0, Color32::from_rgb(12, 16, 24));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(35, 45, 60)), StrokeKind::Inside);

        let rail_y_top = rect.top() + height * 0.32;
        let ground_y = rect.top() + height * 0.78;

        // Draw ground rail
        painter.line_segment(
            [pos2(rect.left() + 20.0, ground_y), pos2(rect.right() - 20.0, ground_y)],
            Stroke::new(2.0, Color32::from_rgb(50, 70, 95)),
        );
        painter.text(
            pos2(rect.right() - 18.0, ground_y),
            egui::Align2::LEFT_CENTER,
            "GND",
            FontId::monospace(10.0),
            Color32::from_rgb(100, 130, 160),
        );

        // Input and Output ports
        let in_x = rect.left() + 25.0;
        let out_x = rect.right() - 25.0;
        painter.line_segment(
            [pos2(in_x - 12.0, rail_y_top), pos2(in_x, rail_y_top)],
            Stroke::new(2.5, Color32::from_rgb(80, 200, 255)),
        );
        painter.text(
            pos2(in_x - 14.0, rail_y_top - 12.0),
            egui::Align2::LEFT_BOTTOM,
            "RF IN",
            FontId::proportional(11.0),
            Color32::from_rgb(80, 200, 255),
        );

        painter.line_segment(
            [pos2(out_x, rail_y_top), pos2(out_x + 12.0, rail_y_top)],
            Stroke::new(2.5, Color32::from_rgb(80, 200, 255)),
        );
        painter.text(
            pos2(out_x + 14.0, rail_y_top - 12.0),
            egui::Align2::RIGHT_BOTTOM,
            "RF OUT",
            FontId::proportional(11.0),
            Color32::from_rgb(80, 200, 255),
        );

        // Number of representative cells to render
        let n_cells_vis = 8;
        let cell_spacing = (out_x - in_x) / (n_cells_vis as f32);

        for i in 0..n_cells_vis {
            let x_start = in_x + i as f32 * cell_spacing;
            let x_node = x_start + cell_spacing * 0.5;
            let x_next = x_start + cell_spacing;

            // Series Josephson Junction Symbol: Wire with centered 'X'
            let junc_cx = (x_start + x_node) * 0.5;
            painter.line_segment(
                [pos2(x_start, rail_y_top), pos2(junc_cx - 8.0, rail_y_top)],
                Stroke::new(1.8, Color32::from_rgb(180, 200, 220)),
            );
            painter.line_segment(
                [pos2(junc_cx + 8.0, rail_y_top), pos2(x_node, rail_y_top)],
                Stroke::new(1.8, Color32::from_rgb(180, 200, 220)),
            );

            // Draw Josephson Junction 'X' symbol inside box
            let junc_box = Rect::from_center_size(pos2(junc_cx, rail_y_top), vec2(16.0, 16.0));
            painter.rect_filled(junc_box, 2.0, Color32::from_rgb(18, 24, 36));
            painter.rect_stroke(junc_box, 2.0, Stroke::new(1.0, Color32::from_rgb(80, 160, 220)), StrokeKind::Inside);

            painter.line_segment(
                [pos2(junc_cx - 5.0, rail_y_top - 5.0), pos2(junc_cx + 5.0, rail_y_top + 5.0)],
                Stroke::new(1.8, Color32::from_rgb(100, 220, 255)),
            );
            painter.line_segment(
                [pos2(junc_cx - 5.0, rail_y_top + 5.0), pos2(junc_cx + 5.0, rail_y_top - 5.0)],
                Stroke::new(1.8, Color32::from_rgb(100, 220, 255)),
            );

            // Wire to next cell
            painter.line_segment(
                [pos2(x_node, rail_y_top), pos2(x_next, rail_y_top)],
                Stroke::new(1.8, Color32::from_rgb(180, 200, 220)),
            );

            // Node dot
            painter.circle_filled(pos2(x_node, rail_y_top), 3.0, Color32::from_rgb(220, 230, 240));

            // Is this cell an RPM stub location? (e.g. every 4 visible cells representing M_cells period)
            let is_rpm_cell = self.rpm_enabled && (i % 4 == 3);

            if is_rpm_cell {
                // RPM Resonant Shunt Stub branch
                painter.line_segment(
                    [pos2(x_node, rail_y_top), pos2(x_node, rail_y_top + 16.0)],
                    Stroke::new(2.0, Color32::from_rgb(255, 180, 40)),
                );

                // RPM Inductor spiral / zigzag
                let ind_top = rail_y_top + 16.0;
                let ind_bot = ind_top + 22.0;
                let zig1 = pos2(x_node - 6.0, ind_top + 6.0);
                let zig2 = pos2(x_node + 6.0, ind_top + 12.0);
                let zig3 = pos2(x_node - 6.0, ind_top + 18.0);
                painter.line_segment([pos2(x_node, ind_top), zig1], Stroke::new(1.8, Color32::from_rgb(255, 180, 40)));
                painter.line_segment([zig1, zig2], Stroke::new(1.8, Color32::from_rgb(255, 180, 40)));
                painter.line_segment([zig2, zig3], Stroke::new(1.8, Color32::from_rgb(255, 180, 40)));
                painter.line_segment([zig3, pos2(x_node, ind_bot)], Stroke::new(1.8, Color32::from_rgb(255, 180, 40)));

                // Wire between L_rpm and C_rpm
                let cap_top = ind_bot + 8.0;
                painter.line_segment([pos2(x_node, ind_bot), pos2(x_node, cap_top)], Stroke::new(1.8, Color32::from_rgb(255, 180, 40)));

                // RPM Capacitor plates
                let plate_w = 14.0;
                painter.line_segment([pos2(x_node - plate_w * 0.5, cap_top), pos2(x_node + plate_w * 0.5, cap_top)], Stroke::new(2.2, Color32::from_rgb(255, 180, 40)));
                painter.line_segment([pos2(x_node - plate_w * 0.5, cap_top + 5.0), pos2(x_node + plate_w * 0.5, cap_top + 5.0)], Stroke::new(2.2, Color32::from_rgb(255, 180, 40)));

                // To ground
                painter.line_segment([pos2(x_node, cap_top + 5.0), pos2(x_node, ground_y)], Stroke::new(1.8, Color32::from_rgb(255, 180, 40)));

                // Label RPM
                painter.text(
                    pos2(x_node + 8.0, ind_top + 10.0),
                    egui::Align2::LEFT_CENTER,
                    "RPM Stub",
                    FontId::monospace(9.0),
                    Color32::from_rgb(255, 200, 60),
                );
            } else {
                // Standard Ground Shunt Capacitor C_g
                let cap_mid_y = (rail_y_top + ground_y) * 0.5;
                painter.line_segment(
                    [pos2(x_node, rail_y_top), pos2(x_node, cap_mid_y - 4.0)],
                    Stroke::new(1.5, Color32::from_rgb(100, 130, 170)),
                );

                // Capacitor plates
                let plate_w = 12.0;
                painter.line_segment(
                    [pos2(x_node - plate_w * 0.5, cap_mid_y - 4.0), pos2(x_node + plate_w * 0.5, cap_mid_y - 4.0)],
                    Stroke::new(2.0, Color32::from_rgb(100, 220, 200)),
                );
                painter.line_segment(
                    [pos2(x_node - plate_w * 0.5, cap_mid_y + 2.0), pos2(x_node + plate_w * 0.5, cap_mid_y + 2.0)],
                    Stroke::new(2.0, Color32::from_rgb(100, 220, 200)),
                );

                // Lower lead to ground
                painter.line_segment(
                    [pos2(x_node, cap_mid_y + 2.0), pos2(x_node, ground_y)],
                    Stroke::new(1.5, Color32::from_rgb(100, 130, 170)),
                );
            }

            // Cell label
            painter.text(
                pos2(x_node, rail_y_top - 10.0),
                egui::Align2::CENTER_BOTTOM,
                format!("#{}", (i + 1) * (self.num_cells / n_cells_vis)),
                FontId::monospace(8.0),
                Color32::from_rgb(140, 160, 180),
            );
        }

        // Legend overlay at bottom
        let legend_y = rect.bottom() - 14.0;
        painter.text(
            pos2(rect.left() + 20.0, legend_y),
            egui::Align2::LEFT_CENTER,
            "L_J0: Josephson Junction  |  C_g: Shunt Cap  |  RPM: Periodic Stub Filter (12 GHz)",
            FontId::proportional(10.0),
            Color32::from_rgb(130, 150, 175),
        );
    }

    /// Renders continuous 4WM gain spectrum G(f) across the octave bandwidth.
    fn render_gain_spectrum_plot(&self, ui: &mut Ui, width: f32, height: f32) {
        Plot::new("jtwpa_gain_spectrum_plot")
            .width(width)
            .height(height)
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                // 20 dB gain target threshold dashed line
                let threshold_line = vec![
                    [self.signal_start_ghz, 20.0],
                    [self.signal_stop_ghz, 20.0],
                ];
                plot_ui.line(
                    Line::new("20 dB Gain Threshold", PlotPoints::new(threshold_line))
                        .color(Color32::from_rgb(255, 215, 0))
                        .stroke(Stroke::new(1.5, Color32::from_rgb(255, 215, 0))),
                );

                // Signal Power Gain G_s(f) curve
                if !self.gain_curve.is_empty() {
                    plot_ui.line(
                        Line::new("Signal Gain G_s (dB)", PlotPoints::new(self.gain_curve.clone()))
                            .color(Color32::from_rgb(20, 240, 210))
                            .stroke(Stroke::new(2.5, Color32::from_rgb(20, 240, 210))),
                    );
                }

                // Idler Conversion Gain G_i(f) curve
                if !self.idler_gain_curve.is_empty() {
                    plot_ui.line(
                        Line::new("Idler Gain G_i (dB)", PlotPoints::new(self.idler_gain_curve.clone()))
                            .color(Color32::from_rgb(255, 140, 40))
                            .stroke(Stroke::new(1.8, Color32::from_rgb(255, 140, 40))),
                    );
                }
            });
    }

    /// Renders phase matching dispersion curve Delta_k(f) with vs without RPM engineering.
    fn render_dispersion_plot(&self, ui: &mut Ui, width: f32, height: f32) {
        Plot::new("jtwpa_dispersion_plot")
            .width(width)
            .height(height)
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                // Zero phase mismatch reference line
                let zero_line = vec![
                    [self.signal_start_ghz, 0.0],
                    [self.signal_stop_ghz, 0.0],
                ];
                plot_ui.line(
                    Line::new("Phase Matched Delta_k = 0", PlotPoints::new(zero_line))
                        .color(Color32::from_rgb(80, 100, 130))
                        .stroke(Stroke::new(1.2, Color32::from_rgb(80, 100, 130))),
                );

                // Bare dispersion mismatch Delta_k without RPM
                if !self.dispersion_bare_curve.is_empty() {
                    plot_ui.line(
                        Line::new("Bare Line (Without RPM)", PlotPoints::new(self.dispersion_bare_curve.clone()))
                            .color(Color32::from_rgb(255, 80, 80))
                            .stroke(Stroke::new(2.0, Color32::from_rgb(255, 80, 80))),
                    );
                }

                // Engineered dispersion mismatch Delta_k with RPM
                if !self.dispersion_rpm_curve.is_empty() {
                    plot_ui.line(
                        Line::new("With RPM Filter Stubs", PlotPoints::new(self.dispersion_rpm_curve.clone()))
                            .color(Color32::from_rgb(40, 220, 120))
                            .stroke(Stroke::new(2.5, Color32::from_rgb(40, 220, 120))),
                    );
                }
            });
    }

    /// Renders quantum quadrature uncertainty ellipse in phase space against SQL.
    fn render_squeezing_canvas(&self, ui: &mut Ui, width: f32, height: f32) {
        let (response, painter) = ui.allocate_painter(vec2(width, height), Sense::hover());
        let rect = response.rect;

        painter.rect_filled(rect, 4.0, Color32::from_rgb(12, 16, 24));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(35, 45, 60)), StrokeKind::Inside);

        let center = rect.center();
        let scale = width.min(height) * 0.35;

        // Coordinate axes
        painter.line_segment(
            [pos2(center.x - scale * 1.1, center.y), pos2(center.x + scale * 1.1, center.y)],
            Stroke::new(1.0, Color32::from_rgb(45, 60, 80)),
        );
        painter.line_segment(
            [pos2(center.x, center.y - scale * 1.1), pos2(center.x, center.y + scale * 1.1)],
            Stroke::new(1.0, Color32::from_rgb(45, 60, 80)),
        );

        painter.text(
            pos2(center.x + scale * 1.15, center.y),
            egui::Align2::LEFT_CENTER,
            "X1",
            FontId::proportional(11.0),
            Color32::from_rgb(140, 170, 200),
        );
        painter.text(
            pos2(center.x, center.y - scale * 1.15),
            egui::Align2::CENTER_BOTTOM,
            "X2",
            FontId::proportional(11.0),
            Color32::from_rgb(140, 170, 200),
        );

        // Standard Quantum Limit (SQL = 0.25 variance, sigma_sql = 0.5)
        let sql_radius = scale * 0.5;
        painter.circle_stroke(
            center,
            sql_radius,
            Stroke::new(1.5, Color32::from_rgb(90, 110, 140)),
        );
        painter.text(
            pos2(center.x + sql_radius * 0.72, center.y - sql_radius * 0.72),
            egui::Align2::LEFT_BOTTOM,
            "SQL (0.25)",
            FontId::monospace(9.0),
            Color32::from_rgb(110, 135, 165),
        );

        // Squeezed state ellipse: semi-minor axis sigma_x, semi-major axis sigma_y
        let sigma_x = self.squeezing.s_xx.sqrt();
        let sigma_y = self.squeezing.s_yy.sqrt();

        // Screen radii
        let rx = (scale * sigma_x as f32).max(4.0);
        let ry = (scale * sigma_y as f32).min(scale * 1.05);

        // Draw filled squeezed ellipse
        let n_pts = 48;
        let mut ellipse_pts = Vec::with_capacity(n_pts);
        for i in 0..n_pts {
            let theta = (i as f32 / n_pts as f32) * std::f32::consts::TAU;
            let px = center.x + rx * theta.cos();
            let py = center.y + ry * theta.sin();
            ellipse_pts.push(pos2(px, py));
        }

        painter.add(egui::Shape::convex_polygon(
            ellipse_pts.clone(),
            Color32::from_rgba_unmultiplied(40, 220, 200, 45),
            Stroke::new(2.2, Color32::from_rgb(40, 220, 200)),
        ));

        // Minor and major axes markers
        painter.line_segment(
            [pos2(center.x - rx, center.y), pos2(center.x + rx, center.y)],
            Stroke::new(1.5, Color32::from_rgb(255, 100, 100)),
        );
        painter.line_segment(
            [pos2(center.x, center.y - ry), pos2(center.x, center.y + ry)],
            Stroke::new(1.5, Color32::from_rgb(100, 255, 180)),
        );

        // Metrics badge
        let badge_rect = Rect::from_min_size(pos2(rect.left() + 10.0, rect.top() + 10.0), vec2(175.0, 68.0));
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgba_unmultiplied(16, 22, 32, 220));
        painter.rect_stroke(badge_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(40, 55, 75)), StrokeKind::Inside);

        painter.text(
            pos2(badge_rect.left() + 8.0, badge_rect.top() + 6.0),
            egui::Align2::LEFT_TOP,
            format!("Squeezing: {:.1} dB", self.squeezing_db),
            FontId::proportional(11.0),
            Color32::from_rgb(40, 220, 200),
        );
        painter.text(
            pos2(badge_rect.left() + 8.0, badge_rect.top() + 22.0),
            egui::Align2::LEFT_TOP,
            format!("S_xx = {:.3e}", self.squeezing.s_xx),
            FontId::monospace(10.0),
            Color32::from_rgb(160, 185, 210),
        );
        painter.text(
            pos2(badge_rect.left() + 8.0, badge_rect.top() + 36.0),
            egui::Align2::LEFT_TOP,
            format!("S_yy = {:.2}", self.squeezing.s_yy),
            FontId::monospace(10.0),
            Color32::from_rgb(160, 185, 210),
        );
        painter.text(
            pos2(badge_rect.left() + 8.0, badge_rect.top() + 50.0),
            egui::Align2::LEFT_TOP,
            format!("n_add = {:.3} quanta", self.n_add),
            FontId::monospace(10.0),
            Color32::from_rgb(255, 215, 0),
        );
    }

    /// Renders telemetry footer with key microwave metrics.
    fn render_telemetry_footer(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(18.0, 0.0);

            // 1. Peak Gain
            ui.vertical(|ui| {
                ui.label(RichText::new("PEAK GAIN").size(9.0).color(Color32::from_rgb(120, 150, 180)));
                let color = if self.peak_gain_db >= 20.0 {
                    Color32::from_rgb(40, 220, 120)
                } else {
                    Color32::from_rgb(255, 180, 40)
                };
                ui.label(RichText::new(format!("{:.2} dB", self.peak_gain_db)).size(13.0).strong().color(color));
            });

            ui.separator();

            // 2. 3-dB Bandwidth
            ui.vertical(|ui| {
                ui.label(RichText::new("3-dB BANDWIDTH").size(9.0).color(Color32::from_rgb(120, 150, 180)));
                ui.label(RichText::new(format!("{:.2} GHz", self.bandwidth_3db_ghz)).size(13.0).strong().color(Color32::from_rgb(100, 220, 255)));
            });

            ui.separator();

            // 3. Squeezing
            ui.vertical(|ui| {
                ui.label(RichText::new("SQUEEZING").size(9.0).color(Color32::from_rgb(120, 150, 180)));
                ui.label(RichText::new(format!("{:.1} dB below SQL", self.squeezing_db)).size(13.0).strong().color(Color32::from_rgb(230, 140, 255)));
            });

            ui.separator();

            // 4. Added Noise Quanta
            ui.vertical(|ui| {
                ui.label(RichText::new("ADDED NOISE").size(9.0).color(Color32::from_rgb(120, 150, 180)));
                ui.label(RichText::new(format!("{:.3} photons", self.n_add)).size(13.0).strong().color(Color32::from_rgb(255, 215, 0)));
            });

            ui.separator();

            // 5. Pump Power
            ui.vertical(|ui| {
                ui.label(RichText::new("PUMP POWER").size(9.0).color(Color32::from_rgb(120, 150, 180)));
                ui.label(RichText::new(format!("{:.1} dBm ({:.1} chip)", self.pump_power_dbm, self.on_chip_power_dbm)).size(13.0).strong().color(Color32::from_rgb(180, 205, 230)));
            });

            ui.separator();

            // 6. Phase Mismatch Delta_k
            ui.vertical(|ui| {
                ui.label(RichText::new("PHASE MISMATCH").size(9.0).color(Color32::from_rgb(120, 150, 180)));
                let color = if self.delta_k_rad_per_m.abs() < 25.0 {
                    Color32::from_rgb(40, 220, 120)
                } else {
                    Color32::from_rgb(255, 90, 90)
                };
                ui.label(RichText::new(format!("{:.1} rad/m", self.delta_k_rad_per_m)).size(13.0).strong().color(color));
            });
        });
    }
}
