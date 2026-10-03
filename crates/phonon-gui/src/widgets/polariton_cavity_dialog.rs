#![deny(unsafe_code)]

//! Interactive Topological Cavity QED & Polariton Waveguide Visualizer modal dialog.
//!
//! Provides 2D lattice spatial mode intensity visualization |\psi(x, y)|^2,
//! chiral boundary circulation arrows, defect toggle with disorder immunity telemetry,
//! multi-mode dispersion diagram (LPB, UPB, and topological edge state), and
//! non-reciprocal S-parameter transmission spectrum with vacuum Rabi splitting.

use egui::{
    pos2, vec2, Color32, FontId, Rect, RichText, Sense, Stroke, Ui, Vec2,
};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use phonon_solver::polariton_waveguide::{
    ChiralEdgeModeSolver, ChiralLatticeDefect, ChiralTransmissionPoint, ChiralWavefunction2D,
    MultiModePolaritonDispersionSolver, PolaritonWaveguideParams,
};

/// Interactive modal dialog for polariton waveguides and topological cavity QED.
#[derive(Debug, Clone, PartialEq)]
pub struct PolaritonCavityDialog {
    /// Window visibility toggle.
    pub is_open: bool,

    // Waveguide & Hamiltonian parameters
    pub params: PolaritonWaveguideParams,
    pub nx: usize,
    pub ny: usize,
    pub defect_enabled: bool,
    pub selected_k_um: f64,
    pub selected_mode_m: usize,

    // Visual display toggles
    pub show_circulation_arrows: bool,
    pub show_dispersion_plot: bool,
    pub show_spectrum_plot: bool,
    pub show_telemetry_panel: bool,

    // Cached simulation results
    pub dispersion_points_lp: Vec<[f64; 2]>,
    pub dispersion_points_up: Vec<[f64; 2]>,
    pub dispersion_points_edge: Vec<[f64; 2]>,
    pub transmission_spectrum: Vec<ChiralTransmissionPoint>,
    pub wavefunction_2d: Option<ChiralWavefunction2D>,

    // Cached telemetry metrics
    pub rabi_splitting_mev: f64,
    pub non_reciprocal_isolation_db: f64,
    pub hopfield_exciton_frac: f64,
    pub hopfield_photon_frac: f64,
    pub defect_transmission_ratio: f64,
    pub t_clean: f64,
    pub t_defect: f64,
    pub insertion_loss_db: f64,
    pub perimeter_confinement: f64,

    pub status_msg: String,
    pub run_requested: bool,
}

impl Default for PolaritonCavityDialog {
    fn default() -> Self {
        let mut dialog = Self {
            is_open: false,
            params: PolaritonWaveguideParams::default(),
            nx: 20,
            ny: 20,
            defect_enabled: false,
            selected_k_um: 0.0,
            selected_mode_m: 0,

            show_circulation_arrows: true,
            show_dispersion_plot: true,
            show_spectrum_plot: true,
            show_telemetry_panel: true,

            dispersion_points_lp: Vec::new(),
            dispersion_points_up: Vec::new(),
            dispersion_points_edge: Vec::new(),
            transmission_spectrum: Vec::new(),
            wavefunction_2d: None,

            rabi_splitting_mev: 30.0,
            non_reciprocal_isolation_db: 37.8,
            hopfield_exciton_frac: 0.5,
            hopfield_photon_frac: 0.5,
            defect_transmission_ratio: 1.0,
            t_clean: 0.982,
            t_defect: 0.982,
            insertion_loss_db: 0.08,
            perimeter_confinement: 0.88,

            status_msg: "Polariton simulator ready.".to_string(),
            run_requested: false,
        };
        dialog.run_simulation();
        dialog
    }
}

impl PolaritonCavityDialog {
    /// Creates a new dialog instance with evaluated default simulation states.
    pub fn new() -> Self {
        Self::default()
    }

    /// Solves multi-mode dispersion, chiral S-parameters, and 2D spatial mode wavefunctions.
    pub fn run_simulation(&mut self) {
        // 1. Solve multi-mode dispersion curves
        let disp_solver = MultiModePolaritonDispersionSolver::new(self.params.clone());
        let pts = disp_solver.sweep_mode(self.selected_mode_m, -4.0, 4.0, 81);

        self.dispersion_points_lp = pts.iter().map(|p| [p.k_um, p.energy_lp_mev]).collect();
        self.dispersion_points_up = pts.iter().map(|p| [p.k_um, p.energy_up_mev]).collect();

        // Topological chiral edge state in the bulk gap connecting lower and upper branches
        let e_x = self.params.bare_exciton_energy_mev;
        let v_edge_slope = 3.75;
        self.dispersion_points_edge = pts
            .iter()
            .map(|p| [p.k_um, (e_x + v_edge_slope * p.k_um).clamp(e_x - 14.5, e_x + 14.5)])
            .collect();

        // Single-point detuning telemetry
        let det_pt = disp_solver.solve_point(self.selected_k_um, self.selected_mode_m);
        self.rabi_splitting_mev = det_pt.splitting_mev;
        self.hopfield_exciton_frac = det_pt.exciton_fraction;
        self.hopfield_photon_frac = det_pt.photon_fraction;

        // 2. Solve chiral edge mode transmission & 2D spatial wavefunction
        let mut edge_solver = ChiralEdgeModeSolver::new(self.params.clone()).with_grid_size(self.nx, self.ny);
        if self.defect_enabled {
            edge_solver.defect = edge_solver.default_perimeter_defect();
        } else {
            edge_solver.defect = ChiralLatticeDefect::None;
        }

        let (t_clean, t_defect, ratio) = edge_solver.compute_transmission_with_defect();
        self.t_clean = t_clean;
        self.t_defect = t_defect;
        self.defect_transmission_ratio = if self.defect_enabled { ratio } else { 1.0 };

        self.non_reciprocal_isolation_db = edge_solver.non_reciprocal_isolation_db();
        let res_pt = edge_solver.evaluate_transmission_at_energy(e_x);
        self.insertion_loss_db = res_pt.insertion_loss_db;

        // Transmission spectrum across [E_x - 35 meV, E_x + 35 meV]
        self.transmission_spectrum =
            edge_solver.sweep_transmission_spectrum(e_x - 35.0, e_x + 35.0, 101);

        // 2D spatial mode wavefunction
        let wf = edge_solver.compute_wavefunction_2d();
        self.perimeter_confinement = wf.perimeter_confinement_ratio;
        self.wavefunction_2d = Some(wf);

        let defect_text = if self.defect_enabled {
            "defect active"
        } else {
            "clean perimeter"
        };
        self.status_msg = format!(
            "Converged: C={}, 2g={:.1} meV, Iso={:.1} dB, T_def/T_clean={:.1}% ({}).",
            self.params.chern_number,
            self.rabi_splitting_mev,
            self.non_reciprocal_isolation_db,
            self.defect_transmission_ratio * 100.0,
            defect_text
        );
    }

    /// Renders the modal window UI.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Polariton Waveguide & Topological Photonic Cavity Simulator")
            .open(&mut is_open)
            .default_size([980.0, 680.0])
            .min_size([750.0, 520.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders the complete dialog contents.
    pub fn render_content(&mut self, ui: &mut Ui) {
        // 1. Top Control Bar: Hamiltonian parameters & Defect toggle
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 4.0);

            ui.label(RichText::new("E_x:").size(11.0).color(Color32::from_rgb(180, 200, 220)));
            let e_x_changed = ui
                .add(
                    egui::DragValue::new(&mut self.params.bare_exciton_energy_mev)
                        .range(1000.0..=2000.0)
                        .speed(1.0)
                        .suffix(" meV"),
                )
                .changed();

            ui.label(RichText::new("E_c0:").size(11.0).color(Color32::from_rgb(180, 200, 220)));
            let e_c0_changed = ui
                .add(
                    egui::DragValue::new(&mut self.params.cavity_cutoff_energy_mev)
                        .range(1000.0..=2000.0)
                        .speed(1.0)
                        .suffix(" meV"),
                )
                .changed();

            ui.label(RichText::new("g:").size(11.0).color(Color32::from_rgb(180, 200, 220)));
            let g_changed = ui
                .add(
                    egui::DragValue::new(&mut self.params.rabi_coupling_g_mev)
                        .range(1.0..=100.0)
                        .speed(0.5)
                        .suffix(" meV"),
                )
                .changed();

            ui.label(RichText::new("W:").size(11.0).color(Color32::from_rgb(180, 200, 220)));
            let w_changed = ui
                .add(
                    egui::DragValue::new(&mut self.params.waveguide_width_um)
                        .range(0.5..=20.0)
                        .speed(0.1)
                        .suffix(" um"),
                )
                .changed();

            ui.label(RichText::new("Chern C:").size(11.0).color(Color32::from_rgb(180, 200, 220)));
            let mut chern_changed = false;
            let c_str = match self.params.chern_number {
                1 => "+1 (Topological)",
                -1 => "-1 (Reversed)",
                _ => "0 (Trivial)",
            };
            egui::ComboBox::from_id_salt("chern_number_combo")
                .selected_text(c_str)
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut self.params.chern_number, 1, "+1 (Topological)").clicked() {
                        chern_changed = true;
                    }
                    if ui.selectable_value(&mut self.params.chern_number, -1, "-1 (Reversed)").clicked() {
                        chern_changed = true;
                    }
                    if ui.selectable_value(&mut self.params.chern_number, 0, "0 (Trivial)").clicked() {
                        chern_changed = true;
                    }
                });

            ui.separator();

            let defect_changed = ui.checkbox(&mut self.defect_enabled, "Defect Obstacle").changed();
            ui.checkbox(&mut self.show_circulation_arrows, "Circulation Arrows");

            if ui.button("Re-solve").clicked() || e_x_changed || e_c0_changed || g_changed || w_changed || chern_changed || defect_changed {
                self.run_simulation();
            }
        });

        ui.separator();

        // 2. Main Workspace: Canvas & Telemetry on Left, Plots on Right
        ui.columns(2, |columns| {
            // Left Column: 2D Spatial Lattice & Telemetry
            columns[0].vertical(|ui| {
                ui.heading(
                    RichText::new("2D Spatial Mode Profile |\u{03c8}(x, y)|^2")
                        .size(13.0)
                        .color(Color32::from_rgb(220, 235, 255)),
                );
                self.render_lattice_canvas(ui);

                ui.add_space(8.0);
                self.render_telemetry_panel(ui);
            });

            // Right Column: Dispersion and S-parameter Plots
            columns[1].vertical(|ui| {
                ui.heading(
                    RichText::new("Polariton Dispersion (LPB, UPB & Edge State)")
                        .size(13.0)
                        .color(Color32::from_rgb(220, 235, 255)),
                );
                self.render_dispersion_plot(ui);

                ui.add_space(8.0);
                ui.heading(
                    RichText::new("Transmission Spectrum S_21 & S_12 (Vacuum Rabi Splitting)")
                        .size(13.0)
                        .color(Color32::from_rgb(220, 235, 255)),
                );
                self.render_spectrum_plot(ui);
            });
        });

        ui.separator();
        ui.label(
            RichText::new(&self.status_msg)
                .size(11.0)
                .color(Color32::from_rgb(140, 170, 200)),
        );
    }

    /// Renders the 2D lattice spatial mode intensity visualizer.
    pub fn render_lattice_canvas(&self, ui: &mut Ui) {
        let canvas_dim = ui.available_width().clamp(220.0, 360.0);
        let (response, painter) = ui.allocate_painter(Vec2::splat(canvas_dim), Sense::hover());
        let rect = response.rect;

        let margin = 14.0f32;
        let draw_w = rect.width() - 2.0 * margin;
        let draw_h = rect.height() - 2.0 * margin;

        let nx = self.nx;
        let ny = self.ny;
        let cell_w = draw_w / (nx as f32);
        let cell_h = draw_h / (ny as f32);

        // Substrate background
        painter.rect_filled(rect, 4.0, Color32::from_rgb(12, 16, 24));
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(40, 55, 80)),
            egui::StrokeKind::Inside,
        );

        let wf_opt = self.wavefunction_2d.as_ref();

        for y in 0..ny {
            for x in 0..nx {
                let cell_x0 = rect.min.x + margin + (x as f32) * cell_w;
                let cell_y0 = rect.min.y + margin + ((ny - 1 - y) as f32) * cell_h;
                let cell_rect = Rect::from_min_size(pos2(cell_x0, cell_y0), vec2(cell_w, cell_h));

                let val = if let Some(wf) = wf_opt {
                    wf.intensity_at(x, y)
                } else {
                    0.0
                };

                let color = sample_mode_color(val as f32);
                painter.rect_filled(cell_rect, 0.0, color);
            }
        }

        // Draw defect obstacle overlay if enabled
        if self.defect_enabled {
            let def_x = nx / 2;
            let _def_y = ny - 1;
            let d_x0 = rect.min.x + margin + ((def_x.saturating_sub(1)) as f32) * cell_w;
            let d_y0 = rect.min.y + margin; // At the top edge
            let def_rect = Rect::from_min_size(pos2(d_x0, d_y0), vec2(cell_w * 2.0, cell_h * 2.0));

            painter.rect_filled(def_rect, 2.0, Color32::from_rgb(180, 30, 30));
            painter.rect_stroke(
                def_rect,
                2.0,
                Stroke::new(1.5, Color32::from_rgb(255, 90, 90)),
                egui::StrokeKind::Inside,
            );
            painter.text(
                def_rect.center(),
                egui::Align2::CENTER_CENTER,
                "DEFECT",
                FontId::proportional(9.0),
                Color32::WHITE,
            );
        }

        // Circulation arrows indicating chiral perimeter flow
        if self.show_circulation_arrows && self.params.chern_number != 0 {
            let solver = ChiralEdgeModeSolver::new(self.params.clone())
                .with_grid_size(nx, ny)
                .with_defect(if self.defect_enabled {
                    ChiralLatticeDefect::CornerObstacle {
                        x: nx / 2,
                        y: ny - 1,
                        size: 2,
                    }
                } else {
                    ChiralLatticeDefect::None
                });

            let arrows = solver.circulation_arrows();
            for arr in arrows {
                let p_start = pos2(
                    rect.min.x + margin + arr[0] * draw_w,
                    rect.min.y + margin + (1.0 - arr[1]) * draw_h,
                );
                let p_end = pos2(
                    rect.min.x + margin + arr[2] * draw_w,
                    rect.min.y + margin + (1.0 - arr[3]) * draw_h,
                );
                painter.line_segment([p_start, p_end], Stroke::new(1.5, Color32::from_rgb(56, 189, 248)));

                // Arrowhead
                let dir = (p_end - p_start).normalized();
                let normal = vec2(-dir.y, dir.x);
                let head_len = 5.0f32;
                let head_base = p_end - dir * head_len;
                let a1 = head_base + normal * 3.0;
                let a2 = head_base - normal * 3.0;
                painter.line_segment([p_end, a1], Stroke::new(1.5, Color32::from_rgb(56, 189, 248)));
                painter.line_segment([p_end, a2], Stroke::new(1.5, Color32::from_rgb(56, 189, 248)));
            }
        }

        // Boundary labels for Port 1 and Port 2
        let p1_pos = pos2(rect.min.x + margin, rect.min.y + margin + draw_h * 0.5);
        painter.circle_filled(p1_pos, 4.0, Color32::from_rgb(34, 197, 94));
        painter.text(
            pos2(p1_pos.x + 6.0, p1_pos.y),
            egui::Align2::LEFT_CENTER,
            "P1",
            FontId::proportional(10.0),
            Color32::from_rgb(34, 197, 94),
        );

        let p2_pos = pos2(rect.min.x + margin + draw_w, rect.min.y + margin + draw_h * 0.5);
        painter.circle_filled(p2_pos, 4.0, Color32::from_rgb(59, 130, 246));
        painter.text(
            pos2(p2_pos.x - 6.0, p2_pos.y),
            egui::Align2::RIGHT_CENTER,
            "P2",
            FontId::proportional(10.0),
            Color32::from_rgb(59, 130, 246),
        );
    }

    /// Renders physical telemetry panel with key quantum topological metrics.
    pub fn render_telemetry_panel(&self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
            ui.heading(
                RichText::new("Quantum Metamaterial Telemetry")
                    .size(11.0)
                    .color(Color32::from_rgb(200, 220, 245)),
            );

            egui::Grid::new("polariton_telemetry_grid")
                .num_columns(2)
                .spacing([12.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Chern Invariant:");
                    let c_text = match self.params.chern_number {
                        1 => RichText::new("C = +1 (Topological)").color(Color32::from_rgb(34, 197, 94)).strong(),
                        -1 => RichText::new("C = -1 (Topological)").color(Color32::from_rgb(56, 189, 248)).strong(),
                        _ => RichText::new("C = 0 (Trivial)").color(Color32::from_rgb(234, 179, 8)).strong(),
                    };
                    ui.label(c_text);
                    ui.end_row();

                    ui.label("Vacuum Rabi Splitting (2g):");
                    ui.label(format!("{:.2} meV", self.rabi_splitting_mev));
                    ui.end_row();

                    ui.label("Non-Reciprocal Isolation:");
                    let iso_color = if self.non_reciprocal_isolation_db >= 30.0 {
                        Color32::from_rgb(34, 197, 94)
                    } else {
                        Color32::from_rgb(234, 179, 8)
                    };
                    ui.label(
                        RichText::new(format!("{:.1} dB", self.non_reciprocal_isolation_db))
                            .color(iso_color)
                            .strong(),
                    );
                    ui.end_row();

                    ui.label("Hopfield Exciton / Photon Fraction:");
                    ui.label(format!(
                        "|X|^2 = {:.1}%, |C|^2 = {:.1}%",
                        self.hopfield_exciton_frac * 100.0,
                        self.hopfield_photon_frac * 100.0
                    ));
                    ui.end_row();

                    ui.label("Defect Transmission Ratio:");
                    let ratio_pct = self.defect_transmission_ratio * 100.0;
                    let ratio_color = if self.defect_transmission_ratio >= 0.90 {
                        Color32::from_rgb(34, 197, 94)
                    } else {
                        Color32::from_rgb(239, 68, 68)
                    };
                    ui.label(
                        RichText::new(format!("{:.1}% (T_def / T_clean)", ratio_pct))
                            .color(ratio_color)
                            .strong(),
                    );
                    ui.end_row();

                    ui.label("Forward Insertion Loss:");
                    ui.label(format!("{:.2} dB", self.insertion_loss_db));
                    ui.end_row();

                    ui.label("Perimeter Confinement:");
                    ui.label(format!("{:.1}%", self.perimeter_confinement * 100.0));
                    ui.end_row();
                });
        });
    }

    /// Renders polariton dispersion curves: LPB, UPB, and topological edge state.
    pub fn render_dispersion_plot(&self, ui: &mut Ui) {
        let plot = Plot::new("polariton_dispersion_plot")
            .legend(Legend::default())
            .x_axis_label("Wavevector k (um^-1)")
            .y_axis_label("Energy (meV)")
            .height(170.0)
            .allow_zoom(true)
            .allow_drag(true);

        plot.show(ui, |plot_ui| {
            let lp_points: PlotPoints = PlotPoints::from(self.dispersion_points_lp.clone());
            let lp_line = Line::new("Lower Branch (LPB)", lp_points)
                .color(Color32::from_rgb(56, 189, 248))
                .width(2.0);
            plot_ui.line(lp_line);

            let up_points: PlotPoints = PlotPoints::from(self.dispersion_points_up.clone());
            let up_line = Line::new("Upper Branch (UPB)", up_points)
                .color(Color32::from_rgb(236, 72, 153))
                .width(2.0);
            plot_ui.line(up_line);

            if self.params.chern_number != 0 && !self.dispersion_points_edge.is_empty() {
                let edge_points: PlotPoints = PlotPoints::from(self.dispersion_points_edge.clone());
                let edge_line = Line::new("Topological Edge Mode", edge_points)
                    .color(Color32::from_rgb(250, 204, 21))
                    .width(2.2);
                plot_ui.line(edge_line);
            }
        });
    }

    /// Renders non-reciprocal S-parameter transmission spectrum S_21 and S_12 in dB.
    pub fn render_spectrum_plot(&self, ui: &mut Ui) {
        let plot = Plot::new("polariton_transmission_spectrum_plot")
            .legend(Legend::default())
            .x_axis_label("Energy (meV)")
            .y_axis_label("Transmission (dB)")
            .height(170.0)
            .allow_zoom(true)
            .allow_drag(true);

        plot.show(ui, |plot_ui| {
            let s21_pts: PlotPoints = self
                .transmission_spectrum
                .iter()
                .map(|p| [p.energy_mev, p.s21_db])
                .collect();
            let s21_line = Line::new("S21 Forward (dB)", s21_pts)
                .color(Color32::from_rgb(34, 197, 94))
                .width(2.0);
            plot_ui.line(s21_line);

            let s12_pts: PlotPoints = self
                .transmission_spectrum
                .iter()
                .map(|p| [p.energy_mev, p.s12_db])
                .collect();
            let s12_line = Line::new("S12 Backward (dB)", s12_pts)
                .color(Color32::from_rgb(239, 68, 68))
                .width(2.0);
            plot_ui.line(s12_line);
        });
    }
}

/// Generates a smooth heatmap color from normalized intensity in [0.0, 1.0].
fn sample_mode_color(u: f32) -> Color32 {
    let t = u.clamp(0.0, 1.0);
    if t < 0.25 {
        let s = t / 0.25;
        Color32::from_rgb(
            (15.0 + s * 15.0) as u8,
            (20.0 + s * 30.0) as u8,
            (35.0 + s * 70.0) as u8,
        )
    } else if t < 0.65 {
        let s = (t - 0.25) / 0.40;
        Color32::from_rgb(
            (30.0 + s * 20.0) as u8,
            (50.0 + s * 120.0) as u8,
            (105.0 + s * 115.0) as u8,
        )
    } else if t < 0.90 {
        let s = (t - 0.65) / 0.25;
        Color32::from_rgb(
            (50.0 + s * 190.0) as u8,
            (170.0 + s * 45.0) as u8,
            (220.0 - s * 170.0) as u8,
        )
    } else {
        let s = (t - 0.90) / 0.10;
        Color32::from_rgb(
            (240.0 + s * 15.0) as u8,
            (215.0 + s * 40.0) as u8,
            (50.0 + s * 205.0) as u8,
        )
    }
}
