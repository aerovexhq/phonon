#![deny(unsafe_code)]

//! Interactive Quantum Acoustic Protected Braiding Lattice & Surface Code Visualizer Studio
//! for Phonon CAD Studio.
//!
//! Provides:
//! - 2D Planar T-Junction Braiding Lattice Canvas: renders planar network of topological acoustic
//!   waveguides with localized Majorana wave packets (gamma_1 ... gamma_8), color-coded parity bonds,
//!   and animated / steppable braid trajectories through T-junction branches.
//! - Surface Code Stabilizer Syndrome Grid: interactive 2D diagram of data qubits and ancilla
//!   plaquettes (Star X in green, Plaquette Z in blue) showing detected syndrome defects and
//!   Minimum-Weight Perfect Matching (MWPM) correction chains.
//! - Non-Abelian Braid Word & Gate Fidelity Panel: target gate selector (Hadamard, Phase S, Pauli X,
//!   Pauli Z, CNOT), compiled braid word sequence (e.g. s1 s2 s1...), fidelity meter (>= 99.9%),
//!   and unitary matrix preview.
//! - Dispersive Parity Readout Spectrum: native egui_plot rendering cavity transmission S_21(f)
//!   showing resolved parity doublet peaks (|0> vs |1>) separated by 2*chi with SNR readout (dB).
//! - Controls: Qubit count, topological gap slider, braid duration slider, physical error rate slider,
//!   "Compile Braid" button, "Step Braid Trajectory" button, "Run Syndrome Cycle" button.
//! - Telemetry Footer: Logical State, Target Gate, Gate Fidelity (%), Readout SNR (dB),
//!   Syndrome Status (Clean / Corrected), Logical Error Rate, Protection Gap (MHz).

use egui::{
    pos2, vec2, Align2, Color32, FontId, ProgressBar, RichText, Sense, Stroke, StrokeKind, Ui,
};
use egui_plot::{Legend, Line, Plot, PlotPoints, VLine};
use phonon_solver::protected_braiding_lattice::{
    CompiledBraidResult, CorrectionResult, MajoranaBraidingParams,
    NonAbelianBraidGenerator, ParityReadout, SurfaceCodeGrid, SyndromeResult, TargetGate,
};

/// Studio theme colors.
const COLOR_TOPO_CYAN: Color32 = Color32::from_rgb(56, 189, 248);
const COLOR_TOPO_EMERALD: Color32 = Color32::from_rgb(52, 211, 153);
const COLOR_MZM_GOLD: Color32 = Color32::from_rgb(250, 204, 21);
const COLOR_DEFECT_ROSE: Color32 = Color32::from_rgb(244, 63, 94);
const COLOR_WAVEGUIDE_BG: Color32 = Color32::from_rgb(30, 41, 59);
const COLOR_WAVEGUIDE_CORE: Color32 = Color32::from_rgb(71, 85, 105);
const COLOR_TEXT_DIM: Color32 = Color32::from_rgb(148, 163, 184);

/// Active visual tab in Quantum Braiding Lattice Studio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BraidingDialogTab {
    #[default]
    StudioOverview,
    PlanarLattice,
    SurfaceCode,
    ParitySpectrum,
    BraidCompiler,
}

impl BraidingDialogTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::StudioOverview => "Studio Overview",
            Self::PlanarLattice => "2D Planar T-Junction Lattice",
            Self::SurfaceCode => "Surface Code Syndrome Grid",
            Self::ParitySpectrum => "Dispersive Parity Spectrum",
            Self::BraidCompiler => "Braid Word Compiler",
        }
    }
}

/// Interactive modal dialog for Quantum Acoustic Protected Braiding Lattice Studio.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumBraidingLatticeDialog {
    /// Modal dialog visibility.
    pub is_open: bool,

    /// Active view tab.
    pub active_tab: BraidingDialogTab,

    /// Majorana braiding physical parameters.
    pub params: MajoranaBraidingParams,

    /// Target quantum gate for compilation.
    pub target_gate: TargetGate,

    /// Physical error rate for surface code syndrome extraction.
    pub physical_error_rate: f64,

    /// Trajectory animation step index (0..4 per elementary braid).
    pub anim_step: usize,

    /// Parity readout measurement state (true for even |0>, false for odd |1>).
    pub parity_readout_even: bool,

    /// Cached non-Abelian braid generator.
    pub generator: NonAbelianBraidGenerator,

    /// Cached gate compilation result.
    pub compiled_result: CompiledBraidResult,

    /// Surface code grid patch (distance d = 3).
    pub surface_grid: SurfaceCodeGrid,

    /// Last syndrome extraction result.
    pub last_syndrome: SyndromeResult,

    /// Last syndrome recovery correction result.
    pub last_correction: CorrectionResult,

    /// Cached logical error rate (P_L).
    pub logical_error_rate_cache: f64,

    /// Dispersive parity readout engine.
    pub parity_readout: ParityReadout,
}

impl Default for QuantumBraidingLatticeDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl QuantumBraidingLatticeDialog {
    /// Creates a new dialog instance with default parameters and compiled gates.
    pub fn new() -> Self {
        Self::new_fast()
    }

    /// Fast cold-boot constructor that defers heavy Monte Carlo error rate evaluation.
    pub fn new_fast() -> Self {
        let params = MajoranaBraidingParams::default();
        let generator = NonAbelianBraidGenerator::new(params.clone());
        let target_gate = TargetGate::Hadamard;
        let compiled_result = generator.compile_gate(target_gate);
        let surface_grid = SurfaceCodeGrid::new();
        let physical_error_rate = 0.01;
        let last_syndrome = surface_grid.extract_syndromes(physical_error_rate);
        let last_correction = surface_grid.decode_and_correct(&last_syndrome);
        let logical_error_rate_cache = 0.0001;
        let parity_readout = ParityReadout::from_params(&params);

        Self {
            is_open: false,
            active_tab: BraidingDialogTab::StudioOverview,
            params,
            target_gate,
            physical_error_rate,
            anim_step: 0,
            parity_readout_even: true,
            generator,
            compiled_result,
            surface_grid,
            last_syndrome,
            last_correction,
            logical_error_rate_cache,
            parity_readout,
        }
    }

    /// Recompiles the target quantum gate and updates physical metrics.
    pub fn compile_current_gate(&mut self) {
        self.generator = NonAbelianBraidGenerator::new(self.params.clone());
        self.compiled_result = self.generator.compile_gate(self.target_gate);
        self.parity_readout = ParityReadout::from_params(&self.params);
    }

    /// Advances the animated Majorana trajectory through the T-junction.
    pub fn step_trajectory(&mut self) {
        self.anim_step = (self.anim_step + 1) % 5;
    }

    /// Executes a fresh syndrome extraction and decoding cycle.
    pub fn run_syndrome_cycle(&mut self) {
        self.last_syndrome = self
            .surface_grid
            .extract_syndromes(self.physical_error_rate);
        self.last_correction = self.surface_grid.decode_and_correct(&self.last_syndrome);
        self.logical_error_rate_cache = self
            .surface_grid
            .evaluate_logical_error_rate(self.physical_error_rate, 250);
    }

    /// Renders the modal window inside Phonon Studio.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new("Quantum Acoustic Protected Braiding Lattice Studio")
            .open(&mut is_open)
            .default_size(vec2(1040.0, 740.0))
            .min_width(850.0)
            .min_height(600.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Compatibility entry point.
    pub fn show(&mut self, ctx: &egui::Context) {
        self.ui(ctx);
    }

    /// Main content layout renderer.
    pub fn render_content(&mut self, ui: &mut Ui) {
        // Tab header bar
        ui.horizontal(|ui| {
            ui.heading("Quantum Braiding Processor");
            ui.separator();
            for tab in [
                BraidingDialogTab::StudioOverview,
                BraidingDialogTab::PlanarLattice,
                BraidingDialogTab::SurfaceCode,
                BraidingDialogTab::ParitySpectrum,
                BraidingDialogTab::BraidCompiler,
            ] {
                if ui
                    .selectable_label(self.active_tab == tab, tab.label())
                    .clicked()
                {
                    self.active_tab = tab;
                }
            }
        });

        ui.separator();

        // Control parameters bar
        self.render_controls_bar(ui);

        ui.separator();

        // Main workspace display
        match self.active_tab {
            BraidingDialogTab::StudioOverview => {
                self.render_overview_grid(ui);
            }
            BraidingDialogTab::PlanarLattice => {
                self.render_planar_lattice_canvas(ui, vec2(ui.available_width(), 460.0));
            }
            BraidingDialogTab::SurfaceCode => {
                self.render_surface_code_grid(ui, vec2(ui.available_width(), 460.0));
            }
            BraidingDialogTab::ParitySpectrum => {
                self.render_parity_spectrum_plot(ui, vec2(ui.available_width(), 460.0));
            }
            BraidingDialogTab::BraidCompiler => {
                self.render_compiler_panel(ui);
            }
        }

        ui.separator();

        // Telemetry footer
        self.render_telemetry_footer(ui);
    }

    /// Controls and action buttons bar.
    fn render_controls_bar(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            // Target Gate selector
            ui.label(RichText::new("Target Gate:").strong());
            let current_name = self.target_gate.name();
            egui::ComboBox::from_id_salt("target_gate_combo")
                .selected_text(current_name)
                .show_ui(ui, |ui| {
                    for gate in [
                        TargetGate::Hadamard,
                        TargetGate::PhaseS,
                        TargetGate::PauliX,
                        TargetGate::PauliZ,
                        TargetGate::Cnot,
                    ] {
                        if ui
                            .selectable_value(&mut self.target_gate, gate, gate.name())
                            .clicked()
                        {
                            self.compile_current_gate();
                        }
                    }
                });

            ui.separator();

            // Qubit count
            ui.label("Qubits:");
            let mut qubits = self.params.qubit_count;
            if ui
                .add(egui::Slider::new(&mut qubits, 1..=4).text("logical"))
                .changed()
            {
                self.params.qubit_count = qubits;
                self.compile_current_gate();
            }

            ui.separator();

            // Topological Gap slider
            ui.label("Gap (MHz):");
            if ui
                .add(
                    egui::Slider::new(&mut self.params.topological_gap_mhz, 1.0..=10.0)
                        .step_by(0.1),
                )
                .changed()
            {
                self.compile_current_gate();
            }

            ui.separator();

            // Braid Duration slider
            ui.label("Duration (ns):");
            if ui
                .add(
                    egui::Slider::new(&mut self.params.braid_duration_ns, 50.0..=300.0)
                        .step_by(5.0),
                )
                .changed()
            {
                self.compile_current_gate();
            }

            ui.separator();

            // Physical Error Rate slider
            ui.label("P_phys:");
            if ui
                .add(
                    egui::Slider::new(&mut self.physical_error_rate, 0.001..=0.05)
                        .step_by(0.001),
                )
                .changed()
            {
                self.run_syndrome_cycle();
            }

            ui.separator();

            // Action buttons
            if ui.button("Compile Braid").clicked() {
                self.compile_current_gate();
            }

            if ui
                .button(format!("Step Trajectory [{}/4]", self.anim_step))
                .clicked()
            {
                self.step_trajectory();
            }

            if ui.button("Run Syndrome Cycle").clicked() {
                self.run_syndrome_cycle();
            }
        });
    }

    /// Studio Overview 2x2 grid view.
    fn render_overview_grid(&mut self, ui: &mut Ui) {
        let avail_w = ui.available_width();
        let col_w = (avail_w - 16.0) / 2.0;
        let card_h = 240.0;

        ui.horizontal(|ui| {
            // Top-Left: Planar Lattice Canvas
            ui.vertical(|ui| {
                ui.label(RichText::new("Topological Acoustic Waveguide Lattice").strong());
                self.render_planar_lattice_canvas(ui, vec2(col_w, card_h));
            });

            // Top-Right: Surface Code Grid
            ui.vertical(|ui| {
                ui.label(RichText::new("Surface Code Stabilizer Grid (d = 3)").strong());
                self.render_surface_code_grid(ui, vec2(col_w, card_h));
            });
        });

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            // Bottom-Left: Braid Compiler Details
            ui.vertical(|ui| {
                ui.label(RichText::new("Non-Abelian Braid Word & Unitary Matrix").strong());
                self.render_compiler_card(ui, vec2(col_w, card_h));
            });

            // Bottom-Right: Parity Readout Spectrum
            ui.vertical(|ui| {
                ui.label(RichText::new("Dispersive Parity Cavity Transmission S_21").strong());
                self.render_parity_spectrum_plot(ui, vec2(col_w, card_h));
            });
        });
    }

    /// 2D Planar T-Junction Braiding Lattice Canvas.
    fn render_planar_lattice_canvas(&mut self, ui: &mut Ui, size: egui::Vec2) {
        let (response, painter) = ui.allocate_painter(size, Sense::hover());
        let rect = response.rect;

        // Background
        painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 23, 42));
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(51, 65, 85)),
            StrokeKind::Inside,
        );

        let qubit_count = self.params.qubit_count.min(2);
        let y_spacing = rect.height() / (qubit_count as f32 + 1.0);

        for q in 0..qubit_count {
            let y_bus = rect.top() + (q as f32 + 1.0) * y_spacing;
            let x_left = rect.left() + 40.0;
            let x_right = rect.right() - 40.0;
            let bus_len = x_right - x_left;

            // Horizontal waveguide bus
            painter.line_segment(
                [pos2(x_left, y_bus), pos2(x_right, y_bus)],
                Stroke::new(8.0, COLOR_WAVEGUIDE_BG),
            );
            painter.line_segment(
                [pos2(x_left, y_bus), pos2(x_right, y_bus)],
                Stroke::new(2.5, COLOR_WAVEGUIDE_CORE),
            );

            // Vertical T-junction branches
            let x_j1 = x_left + bus_len * 0.35;
            let x_j2 = x_left + bus_len * 0.65;
            let branch_h = 36.0;

            painter.line_segment(
                [pos2(x_j1, y_bus), pos2(x_j1, y_bus - branch_h)],
                Stroke::new(8.0, COLOR_WAVEGUIDE_BG),
            );
            painter.line_segment(
                [pos2(x_j1, y_bus), pos2(x_j1, y_bus - branch_h)],
                Stroke::new(2.5, COLOR_WAVEGUIDE_CORE),
            );

            painter.line_segment(
                [pos2(x_j2, y_bus), pos2(x_j2, y_bus + branch_h)],
                Stroke::new(8.0, COLOR_WAVEGUIDE_BG),
            );
            painter.line_segment(
                [pos2(x_j2, y_bus), pos2(x_j2, y_bus + branch_h)],
                Stroke::new(2.5, COLOR_WAVEGUIDE_CORE),
            );

            // Compute animated Majorana positions for Qubit 0
            let m_base = q * 4;
            let mut p1 = pos2(x_left + 15.0, y_bus);
            let mut p2 = pos2(x_j1, y_bus);
            let p3 = pos2(x_j2, y_bus);
            let p4 = pos2(x_right - 15.0, y_bus);

            // If qubit 0 and animating, display exchange trajectory of gamma_1 and gamma_2
            if q == 0 {
                match self.anim_step {
                    1 => {
                        // gamma_1 parked in vertical branch
                        p1 = pos2(x_j1, y_bus - branch_h);
                    }
                    2 => {
                        // gamma_1 parked up, gamma_2 moves left past junction
                        p1 = pos2(x_j1, y_bus - branch_h);
                        p2 = pos2(x_left + 15.0, y_bus);
                    }
                    3 => {
                        // gamma_2 at left, gamma_1 moves down to original gamma_2 slot
                        p1 = pos2(x_j1, y_bus);
                        p2 = pos2(x_left + 15.0, y_bus);
                    }
                    4 => {
                        // Exchanged state
                        p1 = pos2(x_j1, y_bus);
                        p2 = pos2(x_left + 15.0, y_bus);
                    }
                    _ => {}
                }
            }

            // Draw color-coded topological parity bonds
            // Pair (gamma_1, gamma_2): Even parity bond in emerald
            painter.line_segment(
                [p1, p2],
                Stroke::new(1.5, COLOR_TOPO_EMERALD.gamma_multiply(0.7)),
            );
            // Pair (gamma_3, gamma_4): Parity bond in cyan
            painter.line_segment(
                [p3, p4],
                Stroke::new(1.5, COLOR_TOPO_CYAN.gamma_multiply(0.7)),
            );

            // Draw 4 Majorana modes
            let modes = [
                (p1, m_base + 1),
                (p2, m_base + 2),
                (p3, m_base + 3),
                (p4, m_base + 4),
            ];

            for (pos, idx) in modes {
                // Outer glow
                painter.circle_filled(
                    pos,
                    11.0,
                    Color32::from_rgba_premultiplied(250, 204, 21, 40),
                );
                // Inner node
                painter.circle_filled(pos, 6.0, COLOR_MZM_GOLD);
                // Center pin
                painter.circle_filled(pos, 2.0, Color32::WHITE);
                // Label
                painter.text(
                    pos + vec2(0.0, 14.0),
                    Align2::CENTER_CENTER,
                    format!("g{}", idx),
                    FontId::monospace(10.0),
                    Color32::WHITE,
                );
            }

            // Qubit register label
            painter.text(
                pos2(rect.left() + 16.0, y_bus - 14.0),
                Align2::LEFT_CENTER,
                format!("Logical Qubit q{}", q),
                FontId::proportional(11.0),
                COLOR_TOPO_CYAN,
            );
        }

        // Stepper overlay hint
        painter.text(
            pos2(rect.right() - 12.0, rect.top() + 12.0),
            Align2::RIGHT_TOP,
            format!("Braid Cycle Step: {}/4", self.anim_step),
            FontId::monospace(10.5),
            COLOR_TEXT_DIM,
        );
    }

    /// Surface Code Stabilizer Syndrome Grid (Distance d = 3).
    fn render_surface_code_grid(&mut self, ui: &mut Ui, size: egui::Vec2) {
        let (response, painter) = ui.allocate_painter(size, Sense::hover());
        let rect = response.rect;

        painter.rect_filled(rect, 4.0, Color32::from_rgb(15, 23, 42));
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(51, 65, 85)),
            StrokeKind::Inside,
        );

        let margin_x = 45.0;
        let margin_y = 35.0;
        let grid_w = rect.width() - 2.0 * margin_x;
        let grid_h = rect.height() - 2.0 * margin_y;

        let scale_x = grid_w / 2.0;
        let scale_y = grid_h / 2.0;

        let map_pos = |x: f64, y: f64| -> egui::Pos2 {
            pos2(
                rect.left() + margin_x + (x as f32) * scale_x,
                rect.top() + margin_y + (y as f32) * scale_y,
            )
        };

        // Render Star (X-type) Stabilizers
        for (idx, check) in self.surface_grid.star_stabilizers.iter().enumerate() {
            let center = map_pos(check.center_x, check.center_y);
            let is_defect = self.last_syndrome.star_defects.contains(&idx);

            let color = if is_defect {
                COLOR_DEFECT_ROSE
            } else {
                COLOR_TOPO_EMERALD
            };

            // Diamond shape for Star stabilizer
            let d_size = 14.0;
            let pts = [
                center + vec2(0.0, -d_size),
                center + vec2(d_size, 0.0),
                center + vec2(0.0, d_size),
                center + vec2(-d_size, 0.0),
            ];
            painter.add(egui::Shape::convex_polygon(
                pts.to_vec(),
                color.gamma_multiply(0.25),
                Stroke::new(1.5, color),
            ));

            painter.text(
                center,
                Align2::CENTER_CENTER,
                format!("A{}", idx + 1),
                FontId::monospace(9.5),
                if is_defect { Color32::WHITE } else { color },
            );
        }

        // Render Plaquette (Z-type) Stabilizers
        for (idx, check) in self.surface_grid.plaquette_stabilizers.iter().enumerate() {
            let center = map_pos(check.center_x, check.center_y);
            let is_defect = self.last_syndrome.plaquette_defects.contains(&idx);

            let color = if is_defect {
                COLOR_DEFECT_ROSE
            } else {
                COLOR_TOPO_CYAN
            };

            // Square shape for Plaquette stabilizer
            let s_size = 12.0;
            let p_rect = egui::Rect::from_center_size(center, vec2(s_size * 2.0, s_size * 2.0));
            painter.rect_filled(p_rect, 2.0, color.gamma_multiply(0.25));
            painter.rect_stroke(p_rect, 2.0, Stroke::new(1.5, color), StrokeKind::Inside);

            painter.text(
                center,
                Align2::CENTER_CENTER,
                format!("B{}", idx + 1),
                FontId::monospace(9.5),
                if is_defect { Color32::WHITE } else { color },
            );
        }

        // Render Correction Chains (MWPM matching paths)
        for &(q_idx, anc_idx) in &self.last_correction.correction_chains {
            let (qx, qy) = self.surface_grid.data_positions[q_idx];
            let q_pos = map_pos(qx, qy);

            let anc_pos = if anc_idx < 4 {
                let check = &self.surface_grid.star_stabilizers[anc_idx];
                map_pos(check.center_x, check.center_y)
            } else {
                let check = &self.surface_grid.plaquette_stabilizers[(anc_idx - 4).min(3)];
                map_pos(check.center_x, check.center_y)
            };

            painter.line_segment(
                [q_pos, anc_pos],
                Stroke::new(2.0, COLOR_DEFECT_ROSE.gamma_multiply(0.8)),
            );
        }

        // Render 9 Data Qubits
        for (idx, &(x, y)) in self.surface_grid.data_positions.iter().enumerate() {
            let pos = map_pos(x, y);
            let has_x = self.last_syndrome.data_x_errors[idx];
            let has_z = self.last_syndrome.data_z_errors[idx];
            let corr_x = self.last_correction.correction_x[idx];
            let corr_z = self.last_correction.correction_z[idx];

            let stroke_color = if has_x || has_z {
                COLOR_DEFECT_ROSE
            } else if corr_x || corr_z {
                COLOR_TOPO_CYAN
            } else {
                Color32::from_rgb(100, 116, 139)
            };

            painter.circle_filled(pos, 8.0, Color32::from_rgb(30, 41, 59));
            painter.circle_stroke(pos, 8.0, Stroke::new(1.8, stroke_color));

            painter.text(
                pos,
                Align2::CENTER_CENTER,
                format!("D{}", idx),
                FontId::monospace(9.0),
                Color32::WHITE,
            );
        }

        // Legend overlay
        painter.text(
            pos2(rect.left() + 10.0, rect.top() + 10.0),
            Align2::LEFT_TOP,
            "Green: Star X (A_s) | Blue: Plaq Z (B_p) | Red: Defect",
            FontId::proportional(10.0),
            COLOR_TEXT_DIM,
        );
    }

    /// Compiler and unitary matrix summary card.
    fn render_compiler_card(&self, ui: &mut Ui, size: egui::Vec2) {
        ui.allocate_ui(size, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Target:").strong());
                        ui.label(self.target_gate.name());
                        ui.separator();
                        ui.label(RichText::new("Fidelity:").strong());
                        let fid_text = format!("{:.3}%", self.compiled_result.net_gate_fidelity * 100.0);
                        ui.colored_label(COLOR_TOPO_EMERALD, fid_text);
                    });

                    ui.add_space(4.0);

                    // Braid word sequence display
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Word:");
                        for step in &self.compiled_result.braid_word {
                            ui.label(RichText::new(step.symbol()).monospace().background_color(Color32::from_rgb(30, 41, 59)));
                        }
                    });

                    ui.add_space(6.0);

                    // Unitary matrix display
                    ui.label(RichText::new("Compiled Unitary Operator U:").strong());
                    let u = &self.compiled_result.unitary;
                    for r in 0..u.dim {
                        ui.horizontal(|ui| {
                            for c in 0..u.dim {
                                let elem = u.get(r, c);
                                let sign = if elem.im >= 0.0 { "+" } else { "-" };
                                let txt = format!("{:+.3}{:}{:0.3}i", elem.re, sign, elem.im.abs());
                                ui.label(RichText::new(txt).monospace().size(10.0));
                            }
                        });
                    }

                    ui.add_space(4.0);
                    ui.label(format!(
                        "Diabatic Error: {:.2e} | Poisoning: {:.2e}",
                        self.compiled_result.diabatic_error, self.compiled_result.poisoning_error
                    ));
                });
            });
        });
    }

    /// Full Braid Word Compiler panel.
    fn render_compiler_panel(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading("Topological Gate Synthesis & Braid Compiler");
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new("Compiled Gate:").strong());
                ui.label(self.target_gate.name());
                ui.separator();
                ui.label(RichText::new("Artin Relation Check:").strong());
                let (artin_ok, res) = self.generator.verify_artin_relation();
                if artin_ok {
                    ui.colored_label(COLOR_TOPO_EMERALD, format!("Verified (norm = {:.1e})", res));
                } else {
                    ui.colored_label(COLOR_DEFECT_ROSE, format!("Residual = {:.1e}", res));
                }
            });

            ui.add_space(8.0);

            // Gate fidelity bar
            let fid = self.compiled_result.net_gate_fidelity;
            ui.label(RichText::new("Process Fidelity (F >= 0.999):").strong());
            ui.add(ProgressBar::new(fid as f32).text(format!("{:.4}%", fid * 100.0)));

            ui.add_space(8.0);

            // Braid word
            ui.label(RichText::new("Compiled Braid Sequence:").strong());
            ui.horizontal_wrapped(|ui| {
                for (idx, step) in self.compiled_result.braid_word.iter().enumerate() {
                    let pill = format!("{}: {}", idx + 1, step.label());
                    ui.label(
                        RichText::new(pill)
                            .monospace()
                            .background_color(Color32::from_rgb(30, 41, 59))
                            .color(COLOR_TOPO_CYAN),
                    );
                }
            });

            ui.add_space(10.0);

            // Unitary Matrix
            ui.label(RichText::new("Synthesized Unitary Matrix U:").strong());
            let u = &self.compiled_result.unitary;
            for r in 0..u.dim {
                ui.horizontal(|ui| {
                    for c in 0..u.dim {
                        let elem = u.get(r, c);
                        let sign = if elem.im >= 0.0 { "+" } else { "-" };
                        let txt = format!("{:+.4} {}{:0.4}i", elem.re, sign, elem.im.abs());
                        ui.label(
                            RichText::new(txt)
                                .monospace()
                                .size(11.0)
                                .background_color(Color32::from_rgb(15, 23, 42)),
                        );
                    }
                });
            }
        });
    }

    /// Dispersive Parity Readout Spectrum Plot.
    fn render_parity_spectrum_plot(&self, ui: &mut Ui, size: egui::Vec2) {
        let (even_raw, odd_raw) = self.parity_readout.generate_spectrum_curves(10.0, 160);
        let even_pts: Vec<[f64; 2]> = even_raw.into_iter().map(|(x, y)| [x, y]).collect();
        let odd_pts: Vec<[f64; 2]> = odd_raw.into_iter().map(|(x, y)| [x, y]).collect();

        let line_even = Line::new("Parity |0> (+chi)", PlotPoints::new(even_pts))
            .color(COLOR_TOPO_EMERALD)
            .width(2.0);

        let line_odd = Line::new("Parity |1> (-chi)", PlotPoints::new(odd_pts))
            .color(COLOR_DEFECT_ROSE)
            .width(2.0);

        let chi = self.parity_readout.dispersive_shift_mhz;
        let vline_even = VLine::new("+chi", chi)
            .color(COLOR_TOPO_EMERALD.gamma_multiply(0.5))
            .width(1.0);
        let vline_odd = VLine::new("-chi", -chi)
            .color(COLOR_DEFECT_ROSE.gamma_multiply(0.5))
            .width(1.0);

        Plot::new("parity_readout_spectrum_plot")
            .width(size.x)
            .height(size.y)
            .legend(Legend::default())
            .x_axis_label("Cavity Detuning f - f_0 (MHz)")
            .y_axis_label("Transmission |S_21|^2")
            .show(ui, |plot_ui| {
                plot_ui.line(line_even);
                plot_ui.line(line_odd);
                plot_ui.vline(vline_even);
                plot_ui.vline(vline_odd);
            });
    }

    /// Telemetry Footer Bar.
    fn render_telemetry_footer(&self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("State:").strong());
            ui.colored_label(COLOR_TOPO_CYAN, "|0>_L");

            ui.separator();

            ui.label(RichText::new("Target:").strong());
            ui.label(self.target_gate.symbol());

            ui.separator();

            ui.label(RichText::new("Fidelity:").strong());
            let fid_str = format!("{:.3}%", self.compiled_result.net_gate_fidelity * 100.0);
            ui.colored_label(COLOR_TOPO_EMERALD, fid_str);

            ui.separator();

            ui.label(RichText::new("Readout SNR:").strong());
            let snr_str = format!("{:.1} dB", self.parity_readout.snr_db());
            ui.colored_label(COLOR_MZM_GOLD, snr_str);

            ui.separator();

            ui.label(RichText::new("Syndromes:").strong());
            if self.last_correction.is_clean {
                ui.colored_label(COLOR_TOPO_EMERALD, "Clean / Corrected");
            } else {
                ui.colored_label(COLOR_DEFECT_ROSE, "Active Defects");
            }

            ui.separator();

            ui.label(RichText::new("P_L:").strong());
            let pl_str = format!("{:.3}%", self.logical_error_rate_cache * 100.0);
            ui.colored_label(COLOR_TOPO_CYAN, pl_str);

            ui.separator();

            ui.label(RichText::new("Protection Gap:").strong());
            let gap_str = format!("{:.2} MHz", self.params.topological_gap_mhz);
            ui.label(gap_str);
        });
    }
}
