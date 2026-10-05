#![deny(unsafe_code)]

//! Interactive Cavity Optomechanical Squeezing, Wigner Function & Phonon Counting Studio Dialog.
//!
//! Provides:
//! - 2D Wigner Function Colormap Heatmap: interactive phase-space canvas rendering W(X, P)
//!   with Turbo / Magma / Cool-Warm colormaps, showing elliptical contours for squeezed states
//!   and deep negative ripples (W < 0 in vibrant blue/violet) for single phonon Fock states.
//! - Phonon Number State Distribution P(n) Bar Chart: native egui_plot bar chart of Fock
//!   state populations n in 0..10.
//! - Quadrature Variance Polar / Angular Scan Curve: egui_plot rendering Delta X_theta^2 vs
//!   theta in [0, pi] with dashed SQL reference line (0.5), showing squeezing dip below SQL.
//! - Dispersive Optomechanical Cavity Number-Resolved Spectrum: egui_plot showing resolved
//!   peaks at omega_c - 2 * n * chi.
//! - Controls & Presets: Mechanical frequency, coupling g0, drive power, thermal bath n_th,
//!   squeezing parameter r. Presets: "Single Phonon Fock State |1>", "Quadrature Squeezed Vacuum",
//!   "Ground State Cooled (n < 0.1)", "Thermal Phonon Bath".
//! - Action buttons: "Calculate Wigner Function", "Sweep Quadrature Angle", "Reset Cavity".
//! - Telemetry Footer: Phonon State, Squeezing Level (dB below SQL), Wigner Negativity W_min,
//!   Mean Phonon Number n_bar, Sub-Poissonian g^(2)(0), Quantum Purity Tr(rho^2), Sideband Cooled n_min.

use egui::{
    pos2, vec2, Align2, Color32, FontId, Rect, RichText, ScrollArea, Sense, Stroke, StrokeKind,
    Ui,
};
use egui_plot::{Bar, BarChart, HLine, Legend, Line, Plot, PlotPoints, VLine};
use phonon_solver::optomechanical_squeezing::{
    FockStateDistribution, NonClassicalityMetrics, OptomechanicalSqueezingParams,
    PhononCountingResolvedSpectrum, PhononStateKind, QuadratureSqueezingSolver,
    QuadratureVariance, WignerQuasiProbability, SQL_VARIANCE,
};

/// Studio theme color palette.
const COLOR_BG: Color32 = Color32::from_rgb(15, 23, 42);
const COLOR_BORDER: Color32 = Color32::from_rgb(51, 65, 85);
const COLOR_SQZ_CYAN: Color32 = Color32::from_rgb(56, 189, 248);
const COLOR_SPECTRUM_EMERALD: Color32 = Color32::from_rgb(52, 211, 153);
const COLOR_SQL_AMBER: Color32 = Color32::from_rgb(251, 191, 36);
const COLOR_FOCK_ROSE: Color32 = Color32::from_rgb(251, 113, 133);
const COLOR_TEXT_DIM: Color32 = Color32::from_rgb(148, 163, 184);
const COLOR_TEXT_BRIGHT: Color32 = Color32::from_rgb(241, 245, 249);

/// Scientific colormap selector for 2D Wigner quasi-probability rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WignerColormap {
    #[default]
    CoolWarm,
    Turbo,
    Magma,
}

impl WignerColormap {
    pub fn label(&self) -> &'static str {
        match self {
            Self::CoolWarm => "Cool-Warm (Diverging)",
            Self::Turbo => "Turbo (Scientific)",
            Self::Magma => "Magma (Perceptual)",
        }
    }
}

/// Visual tabs for the Optomechanical Squeezing Studio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OptomechDialogTab {
    #[default]
    StudioOverview,
    WignerPhaseSpace,
    FockDistribution,
    QuadratureScan,
    DispersiveSpectrum,
}

impl OptomechDialogTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::StudioOverview => "Studio Overview",
            Self::WignerPhaseSpace => "Wigner Phase Space",
            Self::FockDistribution => "Phonon Fock P(n)",
            Self::QuadratureScan => "Quadrature Squeezing",
            Self::DispersiveSpectrum => "Dispersive Spectrum",
        }
    }
}

/// Simulation preset configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OptomechPreset {
    #[default]
    SqueezedVacuum,
    SinglePhononFock1,
    GroundStateCooled,
    ThermalBath,
}

impl OptomechPreset {
    pub fn label(&self) -> &'static str {
        match self {
            Self::SqueezedVacuum => "Quadrature Squeezed Vacuum",
            Self::SinglePhononFock1 => "Single Phonon Fock State |1>",
            Self::GroundStateCooled => "Ground State Cooled (n < 0.1)",
            Self::ThermalBath => "Thermal Phonon Bath",
        }
    }
}

/// Optomechanical squeezing and phonon counting visual studio dialog.
#[derive(Debug, Clone)]
pub struct OptomechanicalSqueezingDialog {
    /// Dialog open/close visibility flag.
    pub is_open: bool,
    /// Active tab in the studio view.
    pub active_tab: OptomechDialogTab,
    /// Active 2D colormap for Wigner function rendering.
    pub colormap: WignerColormap,
    /// Currently selected preset.
    pub selected_preset: OptomechPreset,

    /// Configurable optomechanical cavity parameters.
    pub params: OptomechanicalSqueezingParams,
    /// Active quantum state kind.
    pub state_kind: PhononStateKind,
    /// Coherent displacement parameter alpha for coherent states.
    pub coherent_alpha: f64,
    /// Dispersive optomechanical shift rate chi in MHz.
    pub dispersive_shift_chi_mhz: f64,
    /// Phase-space 2D grid resolution along each axis.
    pub wigner_grid_size: usize,
    /// Phase-space half-width range [-range, range].
    pub wigner_range: f64,

    /// Cached quadrature variance metrics.
    pub variance: QuadratureVariance,
    /// Cached angular quadrature variance scan curve (theta, Delta X_theta^2).
    pub quadrature_scan: Vec<(f64, f64)>,
    /// Cached 2D Wigner quasi-probability distribution.
    pub wigner: WignerQuasiProbability,
    /// Cached phonon number state distribution P(n).
    pub fock_dist: FockStateDistribution,
    /// Cached dispersive cavity number-resolved transmission spectrum S_21(omega).
    pub resolved_spectrum: PhononCountingResolvedSpectrum,
    /// Cached non-classicality and cooling metrics.
    pub metrics: NonClassicalityMetrics,

    /// Informational status message displayed in banner.
    pub status_message: String,
    /// Sweep animation active flag.
    pub is_sweeping_angle: bool,
}

impl Default for OptomechanicalSqueezingDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl OptomechanicalSqueezingDialog {
    /// Creates a new instance of the dialog with default physical parameters.
    pub fn new() -> Self {
        let params = OptomechanicalSqueezingParams::preset_squeezed_vacuum();
        let state_kind = PhononStateKind::SqueezedVacuum;
        let coherent_alpha = 1.0;
        let dispersive_shift_chi_mhz = 5.0;
        let wigner_grid_size = 51;
        let wigner_range = 4.0;

        let solver = QuadratureSqueezingSolver::new(params.clone());
        let variance = solver.solve_variance();
        let quadrature_scan = solver.compute_quadrature_scan_half(48);

        let fock_dist = FockStateDistribution::for_state(
            state_kind,
            params.squeezing_parameter_r,
            params.thermal_phonon_n_th,
            coherent_alpha,
            12,
        );

        let wigner = WignerQuasiProbability::compute_squeezed_vacuum(
            params.squeezing_parameter_r,
            params.quadrature_angle_rad,
            wigner_grid_size,
            wigner_range,
        );

        let resolved_spectrum = PhononCountingResolvedSpectrum::compute(
            params.cavity_freq_ghz,
            dispersive_shift_chi_mhz,
            params.cavity_decay_kappa_mhz,
            &fock_dist,
            240,
        );

        let metrics = NonClassicalityMetrics::evaluate(
            &fock_dist,
            params.mech_damping_gamma_hz,
            params.thermal_phonon_n_th,
            params.optomechanical_damping_rate_hz(),
            params.cavity_decay_kappa_mhz,
            params.mech_freq_mhz,
        );

        Self {
            is_open: false,
            active_tab: OptomechDialogTab::StudioOverview,
            colormap: WignerColormap::CoolWarm,
            selected_preset: OptomechPreset::SqueezedVacuum,
            params,
            state_kind,
            coherent_alpha,
            dispersive_shift_chi_mhz,
            wigner_grid_size,
            wigner_range,
            variance,
            quadrature_scan,
            wigner,
            fock_dist,
            resolved_spectrum,
            metrics,
            status_message: "Optomechanical squeezing engine ready. Two-tone BAE regime active."
                .to_string(),
            is_sweeping_angle: false,
        }
    }

    /// Applies a preset configuration and immediately recomputes all quantum states.
    pub fn apply_preset(&mut self, preset: OptomechPreset) {
        self.selected_preset = preset;
        match preset {
            OptomechPreset::SqueezedVacuum => {
                self.params = OptomechanicalSqueezingParams::preset_squeezed_vacuum();
                self.state_kind = PhononStateKind::SqueezedVacuum;
                self.status_message =
                    "Preset loaded: Quadrature Squeezed Vacuum with two-tone BAE drive."
                        .to_string();
            }
            OptomechPreset::SinglePhononFock1 => {
                self.params = OptomechanicalSqueezingParams::preset_single_phonon_fock();
                self.state_kind = PhononStateKind::SinglePhononFock1;
                self.status_message =
                    "Preset loaded: Single Phonon Fock State |1> with W < 0 negative core."
                        .to_string();
            }
            OptomechPreset::GroundStateCooled => {
                self.params = OptomechanicalSqueezingParams::preset_ground_state_cooled();
                self.state_kind = PhononStateKind::GroundState;
                self.status_message =
                    "Preset loaded: Deep Sideband Cooling into quantum ground state (n < 0.10)."
                        .to_string();
            }
            OptomechPreset::ThermalBath => {
                self.params = OptomechanicalSqueezingParams::preset_thermal_bath();
                self.state_kind = PhononStateKind::ThermalState;
                self.status_message =
                    "Preset loaded: Thermal Phonon Bath with Bose-Einstein distribution."
                        .to_string();
            }
        }
        self.recompute_all();
    }

    /// Recomputes all physics solvers, Wigner distributions, Fock statistics, and spectra.
    pub fn recompute_all(&mut self) {
        let solver = QuadratureSqueezingSolver::new(self.params.clone());
        self.variance = solver.solve_variance();
        self.quadrature_scan = solver.compute_quadrature_scan_half(48);

        self.fock_dist = FockStateDistribution::for_state(
            self.state_kind,
            self.params.squeezing_parameter_r,
            self.params.thermal_phonon_n_th,
            self.coherent_alpha,
            12,
        );

        self.wigner = match self.state_kind {
            PhononStateKind::SinglePhononFock1 => {
                WignerQuasiProbability::compute_single_phonon_fock1(
                    self.wigner_grid_size,
                    self.wigner_range,
                )
            }
            PhononStateKind::GroundState => WignerQuasiProbability::compute_ground_state(
                self.wigner_grid_size,
                self.wigner_range,
            ),
            PhononStateKind::SqueezedVacuum => WignerQuasiProbability::compute_squeezed_vacuum(
                self.params.squeezing_parameter_r,
                self.params.quadrature_angle_rad,
                self.wigner_grid_size,
                self.wigner_range,
            ),
            PhononStateKind::ThermalState => WignerQuasiProbability::compute_thermal_state(
                self.params.thermal_phonon_n_th,
                self.wigner_grid_size,
                self.wigner_range,
            ),
            PhononStateKind::CoherentState => WignerQuasiProbability::compute_schrodinger_cat(
                self.coherent_alpha,
                self.wigner_grid_size,
                self.wigner_range,
            ),
        };

        self.resolved_spectrum = PhononCountingResolvedSpectrum::compute(
            self.params.cavity_freq_ghz,
            self.dispersive_shift_chi_mhz,
            self.params.cavity_decay_kappa_mhz,
            &self.fock_dist,
            240,
        );

        self.metrics = NonClassicalityMetrics::evaluate(
            &self.fock_dist,
            self.params.mech_damping_gamma_hz,
            self.params.thermal_phonon_n_th,
            self.params.optomechanical_damping_rate_hz(),
            self.params.cavity_decay_kappa_mhz,
            self.params.mech_freq_mhz,
        );
    }

    /// Sweeps the quadrature angle theta by 0.1 radians.
    pub fn sweep_quadrature_angle(&mut self) {
        self.params.quadrature_angle_rad =
            (self.params.quadrature_angle_rad + 0.1) % std::f64::consts::PI;
        self.recompute_all();
        self.status_message = format!(
            "Quadrature angle swept to {:.2} rad ({:.1} deg).",
            self.params.quadrature_angle_rad,
            self.params.quadrature_angle_rad.to_degrees()
        );
    }

    /// Resets all optomechanical cavity parameters to defaults.
    pub fn reset_cavity(&mut self) {
        self.params = OptomechanicalSqueezingParams::default();
        self.state_kind = PhononStateKind::SqueezedVacuum;
        self.selected_preset = OptomechPreset::SqueezedVacuum;
        self.coherent_alpha = 1.0;
        self.dispersive_shift_chi_mhz = 5.0;
        self.status_message = "Cavity parameters reset to default baseline.".to_string();
        self.recompute_all();
    }

    /// Renders the primary modal dialog window in the egui context.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        egui::Window::new(
            RichText::new("Cavity Optomechanical Squeezing & Phonon Counting Studio")
                .strong()
                .color(COLOR_SQZ_CYAN),
        )
        .open(&mut is_open)
        .default_width(1080.0)
        .default_height(780.0)
        .min_width(850.0)
        .min_height(600.0)
        .resizable(true)
        .show(ctx, |ui| {
            self.render_content(ui);
        });
        self.is_open = is_open;
    }

    /// Renders the interior dialog components.
    pub fn render_content(&mut self, ui: &mut Ui) {
        // 1. Top Controls & Header Bar
        self.render_header_and_controls(ui);

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(6.0);

        // 2. Navigation Tabs Bar
        self.render_tab_bar(ui);

        ui.add_space(6.0);

        // 3. Main Visual Content Area
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| match self.active_tab {
                OptomechDialogTab::StudioOverview => self.render_studio_overview(ui),
                OptomechDialogTab::WignerPhaseSpace => self.render_wigner_tab(ui),
                OptomechDialogTab::FockDistribution => self.render_fock_tab(ui),
                OptomechDialogTab::QuadratureScan => self.render_quadrature_tab(ui),
                OptomechDialogTab::DispersiveSpectrum => self.render_spectrum_tab(ui),
            });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        // 4. Telemetry Footer
        self.render_telemetry_footer(ui);
    }

    /// Renders the top control banner with presets and action buttons.
    fn render_header_and_controls(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new("Presets:")
                    .color(COLOR_TEXT_BRIGHT)
                    .strong(),
            );

            for &p in &[
                OptomechPreset::SqueezedVacuum,
                OptomechPreset::SinglePhononFock1,
                OptomechPreset::GroundStateCooled,
                OptomechPreset::ThermalBath,
            ] {
                let selected = self.selected_preset == p;
                let btn = egui::Button::new(
                    RichText::new(p.label())
                        .size(11.5)
                        .color(if selected { COLOR_SQZ_CYAN } else { COLOR_TEXT_DIM }),
                );
                if ui.add(btn).clicked() {
                    self.apply_preset(p);
                }
            }

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(16.0);

            // Action buttons
            if ui
                .button(
                    RichText::new("Calculate Wigner Function")
                        .color(COLOR_SPECTRUM_EMERALD)
                        .strong(),
                )
                .clicked()
            {
                self.recompute_all();
                self.status_message = "2D Wigner function recomputed successfully.".to_string();
            }

            if ui
                .button(
                    RichText::new("Sweep Quadrature Angle")
                        .color(COLOR_SQL_AMBER)
                        .strong(),
                )
                .clicked()
            {
                self.sweep_quadrature_angle();
            }

            if ui
                .button(
                    RichText::new("Reset Cavity")
                        .color(COLOR_FOCK_ROSE)
                        .strong(),
                )
                .clicked()
            {
                self.reset_cavity();
            }
        });

        ui.add_space(6.0);

        // Parameter Sliders Bar
        ui.horizontal_wrapped(|ui| {
            let mut changed = false;

            ui.label(RichText::new("Omega_m:").color(COLOR_TEXT_DIM).size(11.0));
            changed |= ui
                .add(
                    egui::DragValue::new(&mut self.params.mech_freq_mhz)
                        .range(1.0..=500.0)
                        .speed(0.5)
                        .suffix(" MHz"),
                )
                .changed();

            ui.label(RichText::new("g_0:").color(COLOR_TEXT_DIM).size(11.0));
            changed |= ui
                .add(
                    egui::DragValue::new(&mut self.params.optomech_coupling_g0_khz)
                        .range(10.0..=5000.0)
                        .speed(10.0)
                        .suffix(" kHz"),
                )
                .changed();

            ui.label(RichText::new("Laser Power:").color(COLOR_TEXT_DIM).size(11.0));
            changed |= ui
                .add(
                    egui::DragValue::new(&mut self.params.drive_power_laser_mw)
                        .range(0.0..=50.0)
                        .speed(0.1)
                        .suffix(" mW"),
                )
                .changed();

            ui.label(RichText::new("Thermal n_th:").color(COLOR_TEXT_DIM).size(11.0));
            changed |= ui
                .add(
                    egui::DragValue::new(&mut self.params.thermal_phonon_n_th)
                        .range(0.0..=200.0)
                        .speed(0.5),
                )
                .changed();

            ui.label(RichText::new("Squeezing r:").color(COLOR_TEXT_DIM).size(11.0));
            changed |= ui
                .add(
                    egui::DragValue::new(&mut self.params.squeezing_parameter_r)
                        .range(0.0..=2.5)
                        .speed(0.02),
                )
                .changed();

            ui.label(RichText::new("Angle theta:").color(COLOR_TEXT_DIM).size(11.0));
            changed |= ui
                .add(
                    egui::DragValue::new(&mut self.params.quadrature_angle_rad)
                        .range(0.0..=std::f64::consts::PI)
                        .speed(0.02)
                        .suffix(" rad"),
                )
                .changed();

            ui.label(RichText::new("Colormap:").color(COLOR_TEXT_DIM).size(11.0));
            egui::ComboBox::from_id_salt("wigner_colormap_combo")
                .selected_text(self.colormap.label())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.colormap,
                        WignerColormap::CoolWarm,
                        WignerColormap::CoolWarm.label(),
                    );
                    ui.selectable_value(
                        &mut self.colormap,
                        WignerColormap::Turbo,
                        WignerColormap::Turbo.label(),
                    );
                    ui.selectable_value(
                        &mut self.colormap,
                        WignerColormap::Magma,
                        WignerColormap::Magma.label(),
                    );
                });

            if changed {
                self.recompute_all();
            }
        });

        // Status Banner
        ui.add_space(4.0);
        ui.label(
            RichText::new(&self.status_message)
                .color(COLOR_TEXT_DIM)
                .italics()
                .size(11.0),
        );
    }

    /// Renders tab selection bar.
    fn render_tab_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for &tab in &[
                OptomechDialogTab::StudioOverview,
                OptomechDialogTab::WignerPhaseSpace,
                OptomechDialogTab::FockDistribution,
                OptomechDialogTab::QuadratureScan,
                OptomechDialogTab::DispersiveSpectrum,
            ] {
                let selected = self.active_tab == tab;
                let text = RichText::new(tab.label()).size(12.0).color(if selected {
                    COLOR_TEXT_BRIGHT
                } else {
                    COLOR_TEXT_DIM
                });

                if ui.selectable_label(selected, text).clicked() {
                    self.active_tab = tab;
                }
            }
        });
    }

    /// Studio Overview: 2x2 grid displaying all 4 core visual engines simultaneously.
    fn render_studio_overview(&mut self, ui: &mut Ui) {
        let avail_w = ui.available_width().max(400.0);
        let col_w = (avail_w - 16.0) * 0.5;
        let sub_h = 240.0;

        ui.horizontal(|ui| {
            // Panel 1: 2D Wigner Function Canvas
            ui.vertical(|ui| {
                ui.set_width(col_w);
                ui.heading(
                    RichText::new("2D Phase-Space Wigner Function W(X, P)")
                        .color(COLOR_SQZ_CYAN)
                        .size(13.0),
                );
                self.render_wigner_canvas(ui, col_w, sub_h);
            });

            ui.add_space(12.0);

            // Panel 2: Phonon Number State Distribution P(n) Bar Chart
            ui.vertical(|ui| {
                ui.set_width(col_w);
                ui.heading(
                    RichText::new("Phonon Number State Distribution P(n)")
                        .color(COLOR_FOCK_ROSE)
                        .size(13.0),
                );
                self.render_fock_barchart(ui, col_w, sub_h);
            });
        });

        ui.add_space(14.0);
        ui.separator();
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            // Panel 3: Quadrature Squeezing Angular Scan
            ui.vertical(|ui| {
                ui.set_width(col_w);
                ui.heading(
                    RichText::new("Quadrature Variance Delta X_theta^2 vs SQL")
                        .color(COLOR_SQL_AMBER)
                        .size(13.0),
                );
                self.render_quadrature_plot(ui, col_w, sub_h);
            });

            ui.add_space(12.0);

            // Panel 4: Dispersive Number-Resolved Cavity Spectrum
            ui.vertical(|ui| {
                ui.set_width(col_w);
                ui.heading(
                    RichText::new("Dispersive Number-Resolved Spectrum S_21(f)")
                        .color(COLOR_SPECTRUM_EMERALD)
                        .size(13.0),
                );
                self.render_spectrum_plot(ui, col_w, sub_h);
            });
        });
    }

    /// Full-width Wigner tab view.
    fn render_wigner_tab(&mut self, ui: &mut Ui) {
        let w = ui.available_width().max(300.0);
        ui.heading(
            RichText::new("2D Phase-Space Wigner Quasi-Probability Distribution")
                .color(COLOR_SQZ_CYAN)
                .size(15.0),
        );
        ui.label(
            RichText::new(
                "Visualizes mechanical phase space (X, P). Non-classical states exhibit strictly negative ripples W(X, P) < 0 (rendered in vibrant violet/blue).",
            )
            .color(COLOR_TEXT_DIM)
            .size(11.5),
        );
        ui.add_space(8.0);
        self.render_wigner_canvas(ui, w, 440.0);
    }

    /// Full-width Fock distribution tab view.
    fn render_fock_tab(&mut self, ui: &mut Ui) {
        let w = ui.available_width().max(300.0);
        ui.heading(
            RichText::new("Phonon Number State Distribution P(n)")
                .color(COLOR_FOCK_ROSE)
                .size(15.0),
        );
        ui.label(
            RichText::new(
                "Evaluates phonon probabilities P(n) for n = 0..10. Squeezed vacuum shows strictly even phonon parity; Fock |1> shows P(1) = 1.0.",
            )
            .color(COLOR_TEXT_DIM)
            .size(11.5),
        );
        ui.add_space(8.0);
        self.render_fock_barchart(ui, w, 440.0);
    }

    /// Full-width Quadrature Squeezing tab view.
    fn render_quadrature_tab(&mut self, ui: &mut Ui) {
        let w = ui.available_width().max(300.0);
        ui.heading(
            RichText::new("Quadrature Variance Scan Delta X_theta^2 vs Angle theta")
                .color(COLOR_SQL_AMBER)
                .size(15.0),
        );
        ui.label(
            RichText::new(
                "Scans variance across quadrature angles theta in [0, pi]. Squeezing below Standard Quantum Limit (SQL = 0.5) exceeds 3.0 dB and up to > 8.0 dB.",
            )
            .color(COLOR_TEXT_DIM)
            .size(11.5),
        );
        ui.add_space(8.0);
        self.render_quadrature_plot(ui, w, 440.0);
    }

    /// Full-width Dispersive Cavity Spectrum tab view.
    fn render_spectrum_tab(&mut self, ui: &mut Ui) {
        let w = ui.available_width().max(300.0);
        ui.heading(
            RichText::new("Strong Dispersive Number-Resolved Cavity Spectrum")
                .color(COLOR_SPECTRUM_EMERALD)
                .size(15.0),
        );
        ui.label(
            RichText::new(
                "When dispersive shift 2*chi >> kappa, cavity transmission resolves distinct peaks at omega_c - 2*n*chi, weighted by P(n).",
            )
            .color(COLOR_TEXT_DIM)
            .size(11.5),
        );
        ui.add_space(8.0);
        self.render_spectrum_plot(ui, w, 440.0);
    }

    /// Renders the 2D Wigner Function Colormap Heatmap on a dedicated egui painter canvas.
    fn render_wigner_canvas(&self, ui: &mut Ui, w: f32, h: f32) {
        let (response, painter) = ui.allocate_painter(vec2(w, h), Sense::hover());
        let rect = response.rect;

        // Background
        painter.rect_filled(rect, 4.0, COLOR_BG);
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, COLOR_BORDER), StrokeKind::Inside);

        let grid_size = self.wigner.grid_size;
        if grid_size == 0 || self.wigner.values.is_empty() {
            return;
        }

        // Reserve margins for coordinate axes and colorbar
        let margin_left = 38.0;
        let margin_right = 44.0;
        let margin_top = 16.0;
        let margin_bottom = 26.0;

        let plot_rect = Rect::from_min_max(
            pos2(rect.min.x + margin_left, rect.min.y + margin_top),
            pos2(rect.max.x - margin_right, rect.max.y - margin_bottom),
        );

        let pw = plot_rect.width();
        let ph = plot_rect.height();
        if pw <= 10.0 || ph <= 10.0 {
            return;
        }

        let cell_w = pw / grid_size as f32;
        let cell_h = ph / grid_size as f32;

        let w_min = self.wigner.w_min;
        let w_max = self.wigner.w_max;

        // Render 2D grid cells
        for ix in 0..grid_size {
            let cx = plot_rect.min.x + ix as f32 * cell_w;
            for ip in 0..grid_size {
                // Invert vertical P-axis so +P is up
                let cy = plot_rect.max.y - (ip + 1) as f32 * cell_h;
                let cell_rect = Rect::from_min_size(pos2(cx, cy), vec2(cell_w + 0.5, cell_h + 0.5));

                let val = self.wigner.at(ix, ip);
                let color = map_wigner_color(val, w_min, w_max, self.colormap);
                painter.rect_filled(cell_rect, 0.0, color);
            }
        }

        // Draw phase space center crosshairs (X=0, P=0)
        let mid_x = plot_rect.min.x + pw * 0.5;
        let mid_y = plot_rect.min.y + ph * 0.5;
        let axis_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 70));
        painter.line_segment([pos2(plot_rect.min.x, mid_y), pos2(plot_rect.max.x, mid_y)], axis_stroke);
        painter.line_segment([pos2(mid_x, plot_rect.min.y), pos2(mid_x, plot_rect.max.y)], axis_stroke);

        // Nodal line overlay for Fock |1>: circle of radius R = 1/sqrt(2) approx 0.7071
        if self.state_kind == PhononStateKind::SinglePhononFock1 {
            let r_nodal = 0.7071 / self.wigner_range;
            let r_px = (r_nodal as f32) * (pw.min(ph) * 0.5);
            painter.circle_stroke(
                pos2(mid_x, mid_y),
                r_px,
                Stroke::new(1.5, Color32::from_rgb(255, 255, 255)),
            );
        }

        // Elliptical contour for Squeezed Vacuum: 1-sigma contour
        if self.state_kind == PhononStateKind::SqueezedVacuum && self.variance.is_squeezed_below_sql() {
            let sig_x = self.variance.var_min.sqrt() / self.wigner_range;
            let sig_p = self.variance.var_max.sqrt() / self.wigner_range;
            let sx_px = (sig_x as f32) * (pw * 0.5);
            let sp_px = (sig_p as f32) * (ph * 0.5);
            let theta = self.variance.theta_min_rad as f32;

            let pts: Vec<egui::Pos2> = (0..=36)
                .map(|i| {
                    let ang = (i as f32 / 36.0) * std::f32::consts::TAU;
                    let u = sx_px * ang.cos();
                    let v = sp_px * ang.sin();
                    let rx = u * theta.cos() - v * theta.sin();
                    let ry = u * theta.sin() + v * theta.cos();
                    pos2(mid_x + rx, mid_y - ry)
                })
                .collect();

            for w in pts.windows(2) {
                painter.line_segment(
                    [w[0], w[1]],
                    Stroke::new(1.5, Color32::from_rgb(250, 204, 21)),
                );
            }
        }

        // Axis labels
        let font_id = FontId::proportional(10.0);
        painter.text(
            pos2(mid_x, rect.max.y - 12.0),
            Align2::CENTER_CENTER,
            "Mechanical X Quadrature",
            font_id.clone(),
            COLOR_TEXT_DIM,
        );
        painter.text(
            pos2(rect.min.x + 12.0, mid_y),
            Align2::CENTER_CENTER,
            "P",
            font_id.clone(),
            COLOR_TEXT_DIM,
        );

        // Colorbar on the right
        let cbar_x = rect.max.x - 28.0;
        let cbar_rect = Rect::from_min_max(
            pos2(cbar_x, plot_rect.min.y),
            pos2(cbar_x + 10.0, plot_rect.max.y),
        );
        painter.rect_stroke(cbar_rect, 1.0, Stroke::new(1.0, COLOR_BORDER), StrokeKind::Inside);

        let cbar_steps = 32;
        let step_h = cbar_rect.height() / cbar_steps as f32;
        for i in 0..cbar_steps {
            let t = (i as f64) / (cbar_steps - 1) as f64;
            let val = w_min + t * (w_max - w_min);
            let col = map_wigner_color(val, w_min, w_max, self.colormap);
            let y = cbar_rect.max.y - (i + 1) as f32 * step_h;
            painter.rect_filled(
                Rect::from_min_size(pos2(cbar_x, y), vec2(10.0, step_h + 0.5)),
                0.0,
                col,
            );
        }

        // Min/max annotations on colorbar
        let text_font = FontId::monospace(9.0);
        painter.text(
            pos2(cbar_x - 3.0, plot_rect.min.y),
            Align2::RIGHT_TOP,
            format!("{:+.2}", w_max),
            text_font.clone(),
            COLOR_TEXT_DIM,
        );
        painter.text(
            pos2(cbar_x - 3.0, plot_rect.max.y),
            Align2::RIGHT_BOTTOM,
            format!("{:+.2}", w_min),
            text_font,
            COLOR_TEXT_DIM,
        );
    }

    /// Renders native egui_plot bar chart of Fock state populations P(n) for n = 0..10.
    fn render_fock_barchart(&self, ui: &mut Ui, w: f32, h: f32) {
        let bars: Vec<Bar> = self
            .fock_dist
            .probabilities
            .iter()
            .enumerate()
            .take(11)
            .map(|(n, &p)| {
                let color = if p > 0.6 {
                    COLOR_SPECTRUM_EMERALD
                } else if p > 0.1 {
                    COLOR_SQZ_CYAN
                } else {
                    Color32::from_rgb(71, 85, 105)
                };
                Bar::new(n as f64, p).width(0.60).fill(color)
            })
            .collect();

        let chart = BarChart::new("phonon_fock_barchart", bars);

        Plot::new("fock_pop_plot")
            .width(w)
            .height(h)
            .show_axes([true, true])
            .show_grid([true, true])
            .include_y(0.0)
            .include_y(1.0)
            .include_x(-0.6)
            .include_x(10.6)
            .legend(Legend::default().position(egui_plot::Corner::RightTop))
            .show(ui, |plot_ui| {
                plot_ui.bar_chart(chart);
                plot_ui.vline(
                    VLine::new("Mean Occupancy <n>", self.fock_dist.mean_phonon_number)
                        .color(COLOR_SQL_AMBER)
                        .style(egui_plot::LineStyle::Dashed { length: 6.0 }),
                );
            });
    }

    /// Renders native egui_plot curve of Delta X_theta^2 vs theta with SQL reference line.
    fn render_quadrature_plot(&self, ui: &mut Ui, w: f32, h: f32) {
        let pts: Vec<[f64; 2]> = self
            .quadrature_scan
            .iter()
            .map(|&(th, v)| [th, v])
            .collect();

        let line = Line::new("Delta X_theta^2", PlotPoints::new(pts))
            .color(COLOR_SQZ_CYAN)
            .width(2.5);

        Plot::new("quadrature_variance_plot")
            .width(w)
            .height(h)
            .show_axes([true, true])
            .show_grid([true, true])
            .include_y(0.0)
            .include_y(1.0)
            .legend(Legend::default().position(egui_plot::Corner::RightTop))
            .show(ui, |plot_ui| {
                plot_ui.line(line);

                // SQL reference line at 0.5
                plot_ui.hline(
                    HLine::new("Standard Quantum Limit (SQL = 0.5)", SQL_VARIANCE)
                        .color(COLOR_SQL_AMBER)
                        .style(egui_plot::LineStyle::Dashed { length: 8.0 }),
                );

                // Marker at angle of maximum squeezing
                plot_ui.vline(
                    VLine::new("Peak Squeezing Angle", self.variance.theta_min_rad)
                        .color(COLOR_SPECTRUM_EMERALD)
                        .style(egui_plot::LineStyle::Dotted { spacing: 4.0 }),
                );
            });
    }

    /// Renders native egui_plot curve of dispersive number-resolved optical spectrum S_21.
    fn render_spectrum_plot(&self, ui: &mut Ui, w: f32, h: f32) {
        let pts: Vec<[f64; 2]> = self
            .resolved_spectrum
            .detunings_mhz
            .iter()
            .zip(&self.resolved_spectrum.spectrum_intensity)
            .map(|(&d, &t)| [d, t])
            .collect();

        let line = Line::new("Dispersive Cavity Response S_21(Delta)", PlotPoints::new(pts))
            .color(COLOR_SPECTRUM_EMERALD)
            .width(2.0);

        Plot::new("dispersive_cavity_plot")
            .width(w)
            .height(h)
            .show_axes([true, true])
            .show_grid([true, true])
            .include_y(0.0)
            .include_y(1.0)
            .legend(Legend::default().position(egui_plot::Corner::RightTop))
            .show(ui, |plot_ui| {
                plot_ui.line(line);

                // Add vertical markers for resolved number peaks n = 0, 1, 2...
                for peak in &self.resolved_spectrum.peaks {
                    if peak.probability > 0.05 {
                        plot_ui.vline(
                            VLine::new(format!("Peak n={}", peak.phonon_number), peak.detuning_mhz)
                                .color(Color32::from_rgba_unmultiplied(251, 191, 36, 120))
                                .style(egui_plot::LineStyle::Dotted { spacing: 3.0 }),
                        );
                    }
                }
            });
    }

    /// Telemetry Footer Bar displaying key physical and non-classical indicators.
    fn render_telemetry_footer(&self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            // State
            ui.label(RichText::new("State:").color(COLOR_TEXT_DIM).size(11.0));
            ui.label(
                RichText::new(self.state_kind.label())
                    .color(COLOR_SQZ_CYAN)
                    .strong()
                    .size(11.0),
            );

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Squeezing Level in dB below SQL
            ui.label(RichText::new("Squeezing:").color(COLOR_TEXT_DIM).size(11.0));
            let sqz_text = if self.variance.is_squeezed_below_sql() {
                format!("{:.2} dB below SQL", self.variance.squeezing_db)
            } else {
                format!("{:.2} dB (SQL / Un-squeezed)", self.variance.squeezing_db)
            };
            ui.label(
                RichText::new(sqz_text)
                    .color(if self.variance.is_squeezed_below_sql() {
                        COLOR_SPECTRUM_EMERALD
                    } else {
                        COLOR_TEXT_DIM
                    })
                    .strong()
                    .size(11.0),
            );

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Wigner Negativity W_min
            ui.label(RichText::new("W_min:").color(COLOR_TEXT_DIM).size(11.0));
            let neg_color = if self.wigner.w_min < -1e-4 {
                COLOR_FOCK_ROSE
            } else {
                COLOR_TEXT_DIM
            };
            let neg_tag = if self.wigner.w_min < -1e-4 {
                "(NON-CLASSICAL)"
            } else {
                "(Gaussian)"
            };
            ui.label(
                RichText::new(format!("{:.4} {}", self.wigner.w_min, neg_tag))
                    .color(neg_color)
                    .strong()
                    .size(11.0),
            );

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Mean Phonon Occupancy <n>
            ui.label(RichText::new("Mean <n>:").color(COLOR_TEXT_DIM).size(11.0));
            ui.label(
                RichText::new(format!("{:.3} phonons", self.fock_dist.mean_phonon_number))
                    .color(COLOR_TEXT_BRIGHT)
                    .size(11.0),
            );

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // g^(2)(0) Correlation
            ui.label(RichText::new("g^(2)(0):").color(COLOR_TEXT_DIM).size(11.0));
            let g2_color = if self.metrics.is_sub_poissonian {
                COLOR_SPECTRUM_EMERALD
            } else {
                COLOR_TEXT_DIM
            };
            let g2_tag = if self.metrics.is_sub_poissonian {
                "(Sub-Poissonian)"
            } else if (self.metrics.second_order_correlation_g2 - 1.0).abs() < 0.05 {
                "(Poissonian)"
            } else {
                "(Super-Poissonian)"
            };
            ui.label(
                RichText::new(format!(
                    "{:.3} {}",
                    self.metrics.second_order_correlation_g2, g2_tag
                ))
                .color(g2_color)
                .strong()
                .size(11.0),
            );

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Quantum Purity Tr(rho^2)
            ui.label(RichText::new("Purity:").color(COLOR_TEXT_DIM).size(11.0));
            ui.label(
                RichText::new(format!("{:.3}", self.metrics.quantum_purity))
                    .color(if self.metrics.is_quantum_pure {
                        COLOR_SQZ_CYAN
                    } else {
                        COLOR_SQL_AMBER
                    })
                    .size(11.0),
            );

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Sideband Cooled n_min
            ui.label(RichText::new("Sideband n_final:").color(COLOR_TEXT_DIM).size(11.0));
            let cooled_str = format!("{:.3}", self.metrics.sideband_cooled_n_final);
            let ground_tag = if self.metrics.sideband_cooled_n_final < 0.10 {
                "(Ground State < 0.10)"
            } else {
                ""
            };
            ui.label(
                RichText::new(format!("{} {}", cooled_str, ground_tag))
                    .color(COLOR_SPECTRUM_EMERALD)
                    .strong()
                    .size(11.0),
            );
        });
    }
}

/// Helper colormap evaluator mapping scalar values to Color32.
fn map_wigner_color(w: f64, w_min: f64, w_max: f64, map: WignerColormap) -> Color32 {
    match map {
        WignerColormap::CoolWarm => {
            if w < 0.0 {
                let neg_depth = (-w_min).max(1e-6);
                let t = ((-w) / neg_depth).clamp(0.0, 1.0) as f32;
                // Cool blue/violet for negative ripples
                let r = (40.0 + 110.0 * t) as u8;
                let g = (60.0 - 30.0 * t) as u8;
                let b = (100.0 + 145.0 * t) as u8;
                Color32::from_rgb(r, g, b)
            } else {
                let pos_height = w_max.max(1e-6);
                let t = (w / pos_height).clamp(0.0, 1.0) as f32;
                // Warm amber/red for positive probability
                let r = (40.0 + 215.0 * t) as u8;
                let g = (60.0 + 110.0 * t) as u8;
                let b = (100.0 - 70.0 * t) as u8;
                Color32::from_rgb(r, g, b)
            }
        }
        WignerColormap::Turbo => {
            let span = (w_max - w_min).max(1e-6);
            let t = ((w - w_min) / span).clamp(0.0, 1.0) as f32;
            let r = (34.61 + t * (1172.0 + t * (1072.0 - t * (1434.0 + t * (2313.0 - t * 1038.0)))))
                .clamp(0.0, 255.0) as u8;
            let g = (23.31 + t * (557.3 + t * (1074.0 - t * (2457.0 - t * (1228.0 - t * 545.0)))))
                .clamp(0.0, 255.0) as u8;
            let b = (27.2 + t * (3211.0 - t * (15327.0 - t * (27814.0 - t * (22569.0 - t * 6832.0)))))
                .clamp(0.0, 255.0) as u8;
            Color32::from_rgb(r, g, b)
        }
        WignerColormap::Magma => {
            let span = (w_max - w_min).max(1e-6);
            let t = ((w - w_min) / span).clamp(0.0, 1.0) as f32;
            let r = (255.0 * (1.1 * t).clamp(0.0, 1.0)) as u8;
            let g = (255.0 * (t.powf(1.8) * 0.9).clamp(0.0, 1.0)) as u8;
            let b = (255.0 * (0.3 + 0.7 * (1.0 - t).powi(2)).clamp(0.0, 1.0)) as u8;
            Color32::from_rgb(r, g, b)
        }
    }
}
