#![deny(unsafe_code)]

//! Interactive Phonon Studio Topological Corner-Induced Acoustic Second-Harmonic
//! Waveguide Interconnect & Nonlinear Frequency Doubler Studio Dialog.
//!
//! Provides a 5-tab topological phononics and non-linear metamaterials CAD studio:
//! 1. "Hierarchical Topology": 2D spatial lattice energy density canvas rendering corner state
//!    localization (Magma/Turbo), bulk gap, and corner confinement % readout.
//! 2. "Frequency Doubler": Second-harmonic conversion efficiency curve eta(P_in) vs fundamental
//!    pump power, transient buildup curve, and spectral purity bar.
//! 3. "Backscattering-Immune Routing": 2D topological boundary wave propagation canvas showing the
//!    frequency-doubled (2*omega_1) wave smoothly traversing a sharp 90-degree corner, with interactive obstacle toggle.
//! 4. "Multi-Octave Beam Steering": Polar/multi-port beam steering diagram directing doubled frequency
//!    into Port 1 / Port 2 / Port 3 with isolation meters.
//! 5. "Audit & Telemetry": 10-point verification checklist (corner confinement >= 85%, overlap integral >= 0.80,
//!    SHG conversion efficiency >= 30%, corner transmission >= 95%, return loss <= -25 dB, defect immunity, cold-boot latency < 2ms).

use egui::{Color32, Context, Pos2, Rect, RichText, Stroke, Ui, Vec2, Window};
use phonon_solver::corner_harmonic_doubler::{
    CornerBendAngle, CornerCouplingParams, CornerToEdgeLattice, CornerToEdgeLatticeResult,
    CornerTopologicalRouter, DoublerParams, DoublerSteadyState, DoublerTransientPoint,
    HarmonicSpectrumPoint, NonlinearFrequencyDoubler, RouterParams, RouterTargetPort,
    RoutingWaveField, ScatteringMatrix,
};

/// Active tab in the Corner Doubler Studio Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CornerDoublerTab {
    HierarchicalTopology,
    FrequencyDoubler,
    BackscatteringImmuneRouting,
    MultiOctaveBeamSteering,
    AuditTelemetry,
}

/// Colormap options for 2D real-space energy density visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogColormap {
    Magma,
    Turbo,
}

/// Mode selection for real-space visualization in Tab 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayModeType {
    Corner0D,
    Edge1D,
}

/// Audit verification criterion entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CornerDoublerAuditCriterion {
    pub criterion: String,
    pub specification: String,
    pub observed_state: String,
    pub is_passed: bool,
    pub technical_notes: String,
}

/// Modal dialog for Phonon Studio Corner Harmonic Doubler & Router.
pub struct CornerDoublerDialog {
    pub is_open: bool,
    pub active_tab: CornerDoublerTab,

    // Parameters
    pub coupling_params: CornerCouplingParams,
    pub doubler_params: DoublerParams,
    pub router_params: RouterParams,

    // Interactive slider bindings
    pub gamma_mhz: f64,
    pub lambda_mhz: f64,
    pub chi_2: f64,
    pub p_in_mw: f64,
    pub q_corner: f64,
    pub g_shg_mhz: f64,
    pub waveguide_length_mm: f64,

    // Visualization options
    pub selected_colormap: DialogColormap,
    pub selected_display_mode: DisplayModeType,

    // Cached simulation results
    pub lattice_result: CornerToEdgeLatticeResult,
    pub steady_state: DoublerSteadyState,
    pub transient_curve: Vec<DoublerTransientPoint>,
    pub efficiency_curve: Vec<(f64, f64)>,
    pub harmonic_spectrum: Vec<HarmonicSpectrumPoint>,
    pub scattering_matrix: ScatteringMatrix,
    pub routing_field: RoutingWaveField,

    // Audit checklist
    pub audit_criteria: Vec<CornerDoublerAuditCriterion>,
    pub audit_score: (usize, usize),
}

impl Default for CornerDoublerDialog {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl CornerDoublerDialog {
    /// Ultra-fast sub-millisecond cold boot constructor (< 2ms boot budget).
    pub fn new_fast() -> Self {
        let cp = CornerCouplingParams::default();
        let dp = DoublerParams::default();
        let rp = RouterParams::default();

        let gamma = cp.gamma_mhz;
        let lambda = cp.lambda_mhz;
        let chi2 = cp.chi_2;
        let p_in = dp.p_in_mw;
        let q = dp.q_corner;
        let g = dp.g_shg_mhz;
        let l_mm = dp.waveguide_length_mm;

        // Use fast analytic evaluations for instant startup
        let lattice = CornerToEdgeLattice::new(cp.clone());
        let lattice_res = lattice.solve_fast();

        let doubler = NonlinearFrequencyDoubler::new(dp.clone());
        let ss = doubler.solve_steady_state();
        let transient = doubler.solve_transient(100.0, 30);
        let eff_curve = doubler.efficiency_curve(1.0, 100.0, 20);
        let spectrum = doubler.harmonic_spectrum();

        let router = CornerTopologicalRouter::new(rp.clone());
        let s_mat = router.solve_scattering_matrix();
        let field = router.compute_wave_field(30);

        let mut dlg = Self {
            is_open: false,
            active_tab: CornerDoublerTab::HierarchicalTopology,
            coupling_params: cp,
            doubler_params: dp,
            router_params: rp,
            gamma_mhz: gamma,
            lambda_mhz: lambda,
            chi_2: chi2,
            p_in_mw: p_in,
            q_corner: q,
            g_shg_mhz: g,
            waveguide_length_mm: l_mm,
            selected_colormap: DialogColormap::Magma,
            selected_display_mode: DisplayModeType::Corner0D,
            lattice_result: lattice_res,
            steady_state: ss,
            transient_curve: transient,
            efficiency_curve: eff_curve,
            harmonic_spectrum: spectrum,
            scattering_matrix: s_mat,
            routing_field: field,
            audit_criteria: Vec::new(),
            audit_score: (10, 10),
        };

        dlg.update_audit_criteria();
        dlg
    }

    /// Full recomputation when parameters are modified.
    pub fn recompute(&mut self) {
        self.coupling_params.gamma_mhz = self.gamma_mhz;
        self.coupling_params.lambda_mhz = self.lambda_mhz;
        self.coupling_params.chi_2 = self.chi_2;

        self.doubler_params.p_in_mw = self.p_in_mw;
        self.doubler_params.q_corner = self.q_corner;
        self.doubler_params.g_shg_mhz = self.g_shg_mhz;
        self.doubler_params.waveguide_length_mm = self.waveguide_length_mm;

        // 1. Solve lattice
        let lattice = CornerToEdgeLattice::new(self.coupling_params.clone());
        self.lattice_result = lattice.solve_fast();

        // 2. Solve doubler
        let doubler = NonlinearFrequencyDoubler::new(self.doubler_params.clone());
        self.steady_state = doubler.solve_steady_state();
        self.transient_curve = doubler.solve_transient(100.0, 30);
        self.efficiency_curve = doubler.efficiency_curve(1.0, 100.0, 20);
        self.harmonic_spectrum = doubler.harmonic_spectrum();

        // 3. Solve router
        let router = CornerTopologicalRouter::new(self.router_params.clone());
        self.scattering_matrix = router.solve_scattering_matrix();
        self.routing_field = router.compute_wave_field(30);

        // 4. Update audit
        self.update_audit_criteria();
    }

    /// Evaluates the 10-point audit checklist.
    pub fn update_audit_criteria(&mut self) {
        let mut criteria = Vec::with_capacity(10);

        // 1. Mid-Gap 0D Corner Mode Energy
        let e_corner = self.lattice_result.corner_energy_mhz;
        let delta_bulk = self.lattice_result.bulk_bandgap_mhz;
        let c1_pass = e_corner.abs() < 0.05 * delta_bulk;
        criteria.push(CornerDoublerAuditCriterion {
            criterion: "Mid-Gap Zero-Energy Corner Mode".to_string(),
            specification: "|E_corner| < 0.05 * Delta_bulk (pinned at mid-gap)".to_string(),
            observed_state: format!("{:.3} MHz (Delta_bulk = {:.1} MHz)", e_corner, delta_bulk),
            is_passed: c1_pass,
            technical_notes: "BBH quadrupole chiral symmetry pins 0D corner eigenstates precisely at zero energy.".to_string(),
        });

        // 2. 0D Corner Confinement Ratio
        let conf = self.lattice_result.corner_confinement;
        let c2_pass = conf >= 0.85;
        criteria.push(CornerDoublerAuditCriterion {
            criterion: "Corner Spatial Confinement Ratio".to_string(),
            specification: "Fraction of modal energy in corner unit cells >= 85.0%".to_string(),
            observed_state: format!("{:.1}%", conf * 100.0),
            is_passed: c2_pass,
            technical_notes: "Topological SOTI localization concentrates acoustic strain into 0D corner resonators.".to_string(),
        });

        // 3. Hierarchical Modal Overlap Integral
        let overlap = self.lattice_result.hierarchical_overlap_integral;
        let c3_pass = overlap >= 0.80;
        criteria.push(CornerDoublerAuditCriterion {
            criterion: "Hierarchical Corner-to-Edge Overlap Integral".to_string(),
            specification: "I_corner_edge = sum |psi_c|^2 * |psi_e| >= 0.800".to_string(),
            observed_state: format!("{:.3}", overlap),
            is_passed: c3_pass,
            technical_notes: "High spatial overlap maximizes nonlinear second-harmonic energy extraction from corner into edge.".to_string(),
        });

        // 4. Topological SOTI Bandgap Protection
        let is_topo = self.coupling_params.is_topological_soti();
        let gap = self.coupling_params.bulk_bandgap_mhz();
        let c4_pass = is_topo && gap >= 8.0;
        criteria.push(CornerDoublerAuditCriterion {
            criterion: "Topological SOTI Bulk Bandgap".to_string(),
            specification: "gamma < lambda and Delta_bulk >= 8.0 MHz".to_string(),
            observed_state: format!("r = {:.2}, Delta_bulk = {:.1} MHz", self.coupling_params.hopping_ratio(), gap),
            is_passed: c4_pass,
            technical_notes: "Quantized quadrupole moment q_xy = 0.5 protects corner nanocavity against bulk dissipation.".to_string(),
        });

        // 5. Second-Harmonic Conversion Efficiency
        let eff = self.steady_state.efficiency;
        let c5_pass = eff >= 0.30;
        criteria.push(CornerDoublerAuditCriterion {
            criterion: "Second-Harmonic Conversion Efficiency".to_string(),
            specification: "eta_doubler = P_out(2*omega) / P_in(omega) >= 30.0%".to_string(),
            observed_state: format!("{:.1}% (P_out = {:.1} mW from P_in = {:.1} mW)", eff * 100.0, self.steady_state.p_out_mw, self.steady_state.p_in_mw),
            is_passed: c5_pass,
            technical_notes: "High-Q corner nanocavity provides resonant enhancement achieving >= 30% second-harmonic yield.".to_string(),
        });

        // 6. Spurious Harmonic Spectral Purity
        let purity = self.steady_state.spectral_purity_db;
        let c6_pass = purity >= 30.0;
        criteria.push(CornerDoublerAuditCriterion {
            criterion: "Spurious Harmonic Spectral Purity".to_string(),
            specification: "Spurious suppression >= 30.0 dB relative to carrier".to_string(),
            observed_state: format!("{:.1} dB", purity),
            is_passed: c6_pass,
            technical_notes: "Evanescent fundamental rejection and phase mismatch suppress fundamental leak and 3rd harmonics.".to_string(),
        });

        // 7. Sharp 90-Degree Corner Transmission
        let t_corner = self.scattering_matrix.s21_transmission_linear;
        let loss_db = self.scattering_matrix.insertion_loss_db;
        let c7_pass = t_corner >= 0.95 && loss_db <= 0.22;
        criteria.push(CornerDoublerAuditCriterion {
            criterion: "Sharp 90-Degree Corner Transmission".to_string(),
            specification: "T_corner >= 95.0% (insertion loss <= 0.22 dB)".to_string(),
            observed_state: format!("{:.1}% ({:.3} dB)", t_corner * 100.0, loss_db),
            is_passed: c7_pass,
            technical_notes: "Topological edge boundary guides 2*omega_1 harmonic around sharp corner with near-zero scattering.".to_string(),
        });

        // 8. Backscattering Return Loss Immunity
        let s11 = self.scattering_matrix.s11_return_loss_db;
        let c8_pass = s11 <= -25.0;
        criteria.push(CornerDoublerAuditCriterion {
            criterion: "Backscattering Return Loss Immunity".to_string(),
            specification: "|S_11|^2 <= -25.0 dB (reflection < 0.316%)".to_string(),
            observed_state: format!("{:.1} dB ({:.3}% reflection)", s11, self.scattering_matrix.s11_reflection_linear * 100.0),
            is_passed: c8_pass,
            technical_notes: "Absence of backscattering modes ensures near-complete suppression of reflected echo waves.".to_string(),
        });

        // 9. Topological Defect / Vacancy Immunity
        let defect_ratio = self.scattering_matrix.defect_immunity_ratio;
        let c9_pass = defect_ratio >= 0.95;
        criteria.push(CornerDoublerAuditCriterion {
            criterion: "Defect / Obstacle Immunity".to_string(),
            specification: "T_defect >= 0.95 * T_clean around vacancy obstacle".to_string(),
            observed_state: format!("{:.1}% (T_defect = {:.1}%, T_clean = {:.1}%)", defect_ratio * 100.0, self.scattering_matrix.defect_transmission_linear * 100.0, self.scattering_matrix.clean_transmission_linear * 100.0),
            is_passed: c9_pass,
            technical_notes: "Topological protection enables the 2*omega_1 wave to smoothly detour around lattice vacancies.".to_string(),
        });

        // 10. Multi-Port Beam Steering Isolation
        let iso = self.scattering_matrix.cross_port_isolation_db;
        let c10_pass = iso >= 25.0;
        criteria.push(CornerDoublerAuditCriterion {
            criterion: "Multi-Port Beam Steering Isolation".to_string(),
            specification: "Cross-port isolation >= 25.0 dB".to_string(),
            observed_state: format!("{:.1} dB ({})", iso, self.router_params.target_port.label()),
            is_passed: c10_pass,
            technical_notes: "Reconfigurable beam steering directs doubled frequency with high extinction ratio into target port.".to_string(),
        });

        let passed = criteria.iter().filter(|c| c.is_passed).count();
        self.audit_score = (passed, criteria.len());
        self.audit_criteria = criteria;
    }

    /// Primary render method for the dialog in egui.
    pub fn ui(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        let mut is_open = self.is_open;
        Window::new("Corner Harmonic Doubler & Router Studio")
            .open(&mut is_open)
            .default_size(Vec2::new(920.0, 680.0))
            .min_size(Vec2::new(760.0, 520.0))
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders inner dialog tabs and contents.
    pub fn render_content(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Topological Acoustic Corner Doubler & Router").color(Color32::from_rgb(120, 210, 255)).size(17.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (pass, total) = self.audit_score;
                let badge_color = if pass == total {
                    Color32::from_rgb(50, 200, 110)
                } else {
                    Color32::from_rgb(240, 160, 40)
                };
                ui.label(RichText::new(format!("Audit: {}/{} PASS", pass, total)).color(badge_color).strong());
            });
        });
        ui.separator();

        // Tab selection bar
        ui.horizontal(|ui| {
            let tabs = [
                (CornerDoublerTab::HierarchicalTopology, "Hierarchical Topology"),
                (CornerDoublerTab::FrequencyDoubler, "Frequency Doubler"),
                (CornerDoublerTab::BackscatteringImmuneRouting, "Backscattering-Immune Routing"),
                (CornerDoublerTab::MultiOctaveBeamSteering, "Multi-Octave Beam Steering"),
                (CornerDoublerTab::AuditTelemetry, "Audit & Telemetry"),
            ];

            for (tab, label) in tabs {
                let is_active = self.active_tab == tab;
                if ui.selectable_label(is_active, label).clicked() {
                    self.active_tab = tab;
                }
            }
        });
        ui.separator();

        // Render selected tab
        egui::ScrollArea::vertical().show(ui, |ui| {
            match self.active_tab {
                CornerDoublerTab::HierarchicalTopology => self.render_tab_topology(ui),
                CornerDoublerTab::FrequencyDoubler => self.render_tab_doubler(ui),
                CornerDoublerTab::BackscatteringImmuneRouting => self.render_tab_routing(ui),
                CornerDoublerTab::MultiOctaveBeamSteering => self.render_tab_beam_steering(ui),
                CornerDoublerTab::AuditTelemetry => self.render_tab_audit(ui),
            }
        });
    }

    // TAB 1: Hierarchical Topology
    fn render_tab_topology(&mut self, ui: &mut Ui) {
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.set_width(260.0);
                ui.label(RichText::new("Lattice Controls").strong().color(Color32::from_rgb(180, 210, 255)));

                ui.label("Intracell Hopping gamma (MHz):");
                changed |= ui.add(egui::Slider::new(&mut self.gamma_mhz, 0.5..=12.0).text("MHz")).changed();

                ui.label("Intercell Hopping lambda (MHz):");
                changed |= ui.add(egui::Slider::new(&mut self.lambda_mhz, 2.0..=16.0).text("MHz")).changed();

                ui.label("Nonlinear Susceptibility chi^(2):");
                changed |= ui.add(egui::Slider::new(&mut self.chi_2, 0.01..=0.30)).changed();

                ui.separator();
                ui.label(RichText::new("Visualization Mode:").strong());
                ui.radio_value(&mut self.selected_display_mode, DisplayModeType::Corner0D, "0D Corner Nanocavity Mode");
                ui.radio_value(&mut self.selected_display_mode, DisplayModeType::Edge1D, "1D Boundary Waveguide Mode");

                ui.separator();
                ui.label(RichText::new("Colormap:").strong());
                ui.radio_value(&mut self.selected_colormap, DialogColormap::Magma, "Magma (High-Contrast)");
                ui.radio_value(&mut self.selected_colormap, DialogColormap::Turbo, "Turbo (Multi-Band)");

                if ui.button("Recompute Eigenmodes").clicked() {
                    changed = true;
                }
            });

            ui.vertical(|ui| {
                // Telemetry cards
                ui.horizontal(|ui| {
                    let r = self.coupling_params.hopping_ratio();
                    let gap = self.coupling_params.bulk_bandgap_mhz();
                    let conf = self.lattice_result.corner_confinement;
                    let overlap = self.lattice_result.hierarchical_overlap_integral;

                    ui.group(|ui| {
                        ui.label(RichText::new("Topological Regime").size(11.0).color(Color32::GRAY));
                        if self.coupling_params.is_topological_soti() {
                            ui.label(RichText::new("SOTI (gamma < lambda)").color(Color32::from_rgb(60, 210, 110)).strong());
                        } else {
                            ui.label(RichText::new("Trivial (gamma > lambda)").color(Color32::LIGHT_RED).strong());
                        }
                        ui.label(format!("Ratio r: {:.3}", r));
                    });

                    ui.group(|ui| {
                        ui.label(RichText::new("Bulk Bandgap").size(11.0).color(Color32::GRAY));
                        ui.label(RichText::new(format!("{:.1} MHz", gap)).color(Color32::from_rgb(100, 200, 255)).strong());
                        ui.label(format!("Delta = 2*|lambda - gamma|"));
                    });

                    ui.group(|ui| {
                        ui.label(RichText::new("Corner Confinement").size(11.0).color(Color32::GRAY));
                        let col = if conf >= 0.85 { Color32::from_rgb(60, 210, 110) } else { Color32::YELLOW };
                        ui.label(RichText::new(format!("{:.1}%", conf * 100.0)).color(col).strong());
                        ui.label(format!("Spec: >= 85.0%"));
                    });

                    ui.group(|ui| {
                        ui.label(RichText::new("Overlap Integral I").size(11.0).color(Color32::GRAY));
                        let col = if overlap >= 0.80 { Color32::from_rgb(60, 210, 110) } else { Color32::YELLOW };
                        ui.label(RichText::new(format!("{:.3}", overlap)).color(col).strong());
                        ui.label(format!("Spec: >= 0.800"));
                    });
                });

                // Canvas rendering real-space lattice energy density
                self.render_lattice_canvas(ui);
            });
        });

        if changed {
            self.recompute();
        }
    }

    fn render_lattice_canvas(&self, ui: &mut Ui) {
        let (rect, _resp) = ui.allocate_exact_size(Vec2::new(480.0, 360.0), egui::Sense::hover());
        let painter = ui.painter();

        painter.rect_filled(rect, 4.0, Color32::from_rgb(14, 18, 25));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 75)), egui::StrokeKind::Inside);

        let grid = match self.selected_display_mode {
            DisplayModeType::Corner0D => &self.lattice_result.corner_intensity_grid,
            DisplayModeType::Edge1D => &self.lattice_result.edge_intensity_grid,
        };

        if grid.is_empty() || grid[0].is_empty() {
            return;
        }

        let sy = grid.len();
        let sx = grid[0].len();

        let pad = 24.0;
        let cell_w = (rect.width() - 2.0 * pad) / (sx as f32);
        let cell_h = (rect.height() - 2.0 * pad) / (sy as f32);

        // Find max for normalization
        let mut max_val = 1e-9_f64;
        for row in grid {
            for &val in row {
                if val > max_val {
                    max_val = val;
                }
            }
        }

        for (iy, row) in grid.iter().enumerate() {
            let y_pos = rect.top() + pad + (iy as f32) * cell_h;
            for (ix, &val) in row.iter().enumerate() {
                let x_pos = rect.left() + pad + (ix as f32) * cell_w;
                let norm = (val / max_val).clamp(0.0, 1.0);
                let color = sample_colormap(norm, self.selected_colormap);

                let cell_rect = Rect::from_min_size(Pos2::new(x_pos, y_pos), Vec2::new(cell_w - 1.5, cell_h - 1.5));
                painter.rect_filled(cell_rect, 2.0, color);
            }
        }

        // Draw corner nanocavity highlights
        let highlight_stroke = Stroke::new(2.0, Color32::from_rgb(255, 220, 60));
        let corner_bounds = [
            Rect::from_min_size(Pos2::new(rect.left() + pad, rect.top() + pad), Vec2::new(cell_w * 2.0, cell_h * 2.0)),
            Rect::from_min_size(Pos2::new(rect.right() - pad - cell_w * 2.0, rect.top() + pad), Vec2::new(cell_w * 2.0, cell_h * 2.0)),
            Rect::from_min_size(Pos2::new(rect.left() + pad, rect.bottom() - pad - cell_h * 2.0), Vec2::new(cell_w * 2.0, cell_h * 2.0)),
            Rect::from_min_size(Pos2::new(rect.right() - pad - cell_w * 2.0, rect.bottom() - pad - cell_h * 2.0), Vec2::new(cell_w * 2.0, cell_h * 2.0)),
        ];
        for b in corner_bounds {
            painter.rect_stroke(b, 2.0, highlight_stroke, egui::StrokeKind::Inside);
        }

        painter.text(
            Pos2::new(rect.left() + 10.0, rect.top() + 8.0),
            egui::Align2::LEFT_TOP,
            format!("2D SOTI Lattice: {} ({})", match self.selected_display_mode { DisplayModeType::Corner0D => "0D Corner Localization", DisplayModeType::Edge1D => "1D Edge Mode" }, match self.selected_colormap { DialogColormap::Magma => "Magma", DialogColormap::Turbo => "Turbo" }),
            egui::FontId::proportional(11.0),
            Color32::from_rgb(200, 220, 255),
        );
    }

    // TAB 2: Frequency Doubler
    fn render_tab_doubler(&mut self, ui: &mut Ui) {
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.set_width(260.0);
                ui.label(RichText::new("Pump & Cavity Controls").strong().color(Color32::from_rgb(180, 210, 255)));

                ui.label("Pump Power P_in (mW):");
                changed |= ui.add(egui::Slider::new(&mut self.p_in_mw, 1.0..=150.0).text("mW")).changed();

                ui.label("Corner Cavity Quality Factor Q:");
                changed |= ui.add(egui::Slider::new(&mut self.q_corner, 1000.0..=15000.0)).changed();

                ui.label("Non-linear Rate g_shg (MHz):");
                changed |= ui.add(egui::Slider::new(&mut self.g_shg_mhz, 2.0..=30.0).text("MHz")).changed();

                ui.label("Interconnect Length (mm):");
                changed |= ui.add(egui::Slider::new(&mut self.waveguide_length_mm, 2.0..=30.0).text("mm")).changed();

                ui.separator();
                ui.label(RichText::new("Doubler Telemetry:").strong());
                ui.label(format!("Fundamental: {:.2} GHz", self.doubler_params.f_1_ghz));
                ui.label(format!("Second-Harmonic: {:.2} GHz", self.doubler_params.f_2_ghz));
                ui.label(format!("Cavity U_fund: {:.2} pJ", self.steady_state.cavity_energy_fundamental_pj));
                ui.label(format!("Cavity U_shg: {:.2} pJ", self.steady_state.cavity_energy_harmonic_pj));
                ui.label(format!("Prop Loss: {:.2} dB", self.steady_state.propagation_loss_db));
            });

            ui.vertical(|ui| {
                // Key metric cards
                ui.horizontal(|ui| {
                    ui.group(|ui| {
                        ui.label(RichText::new("Generated SHG Output").size(11.0).color(Color32::GRAY));
                        ui.label(RichText::new(format!("{:.2} mW", self.steady_state.p_out_mw)).color(Color32::from_rgb(100, 230, 255)).strong().size(15.0));
                    });

                    ui.group(|ui| {
                        ui.label(RichText::new("Conversion Efficiency").size(11.0).color(Color32::GRAY));
                        let col = if self.steady_state.efficiency >= 0.30 { Color32::from_rgb(60, 210, 110) } else { Color32::YELLOW };
                        ui.label(RichText::new(format!("{:.1}%", self.steady_state.efficiency_pct)).color(col).strong().size(15.0));
                        ui.label("Target: >= 30.0%");
                    });

                    ui.group(|ui| {
                        ui.label(RichText::new("Spectral Purity").size(11.0).color(Color32::GRAY));
                        let col = if self.steady_state.spectral_purity_db >= 30.0 { Color32::from_rgb(60, 210, 110) } else { Color32::YELLOW };
                        ui.label(RichText::new(format!("{:.1} dB", self.steady_state.spectral_purity_db)).color(col).strong().size(15.0));
                        ui.label("Target: >= 30.0 dB");
                    });
                });

                // Curves
                ui.horizontal(|ui| {
                    self.render_efficiency_curve_canvas(ui);
                    self.render_transient_canvas(ui);
                });

                // Spectral purity bar
                self.render_spectral_purity_bar(ui);
            });
        });

        if changed {
            self.recompute();
        }
    }

    fn render_efficiency_curve_canvas(&self, ui: &mut Ui) {
        let (rect, _resp) = ui.allocate_exact_size(Vec2::new(260.0, 180.0), egui::Sense::hover());
        let painter = ui.painter();

        painter.rect_filled(rect, 4.0, Color32::from_rgb(14, 18, 25));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 75)), egui::StrokeKind::Inside);

        painter.text(Pos2::new(rect.left() + 8.0, rect.top() + 6.0), egui::Align2::LEFT_TOP, "eta(P_in) vs Pump Power", egui::FontId::proportional(11.0), Color32::from_rgb(180, 200, 230));

        let pad_l = 30.0;
        let pad_r = 15.0;
        let pad_b = 24.0;
        let pad_t = 24.0;

        let plot_w = rect.width() - pad_l - pad_r;
        let plot_h = rect.height() - pad_t - pad_b;

        // Draw 30% target reference line
        let y_ref = rect.bottom() - pad_b - (0.30 / 0.60) * plot_h;
        painter.line_segment([Pos2::new(rect.left() + pad_l, y_ref), Pos2::new(rect.right() - pad_r, y_ref)], Stroke::new(1.0, Color32::from_rgb(200, 60, 60)));

        if self.efficiency_curve.len() >= 2 {
            let max_p = self.efficiency_curve.last().unwrap().0.max(10.0);
            let mut points = Vec::new();
            for &(p, eff) in &self.efficiency_curve {
                let x = rect.left() + pad_l + ((p / max_p) as f32) * plot_w;
                let y = rect.bottom() - pad_b - ((eff / 0.60).clamp(0.0, 1.0) as f32) * plot_h;
                points.push(Pos2::new(x, y));
            }

            for i in 0..points.len() - 1 {
                painter.line_segment([points[i], points[i + 1]], Stroke::new(2.0, Color32::from_rgb(80, 210, 255)));
            }
        }

        // Current operating point
        let curr_p = self.p_in_mw.min(100.0);
        let curr_eff = self.steady_state.efficiency;
        let cur_x = rect.left() + pad_l + ((curr_p / 100.0) as f32) * plot_w;
        let cur_y = rect.bottom() - pad_b - ((curr_eff / 0.60).clamp(0.0, 1.0) as f32) * plot_h;
        painter.circle_filled(Pos2::new(cur_x, cur_y), 4.5, Color32::from_rgb(255, 210, 50));
    }

    fn render_transient_canvas(&self, ui: &mut Ui) {
        let (rect, _resp) = ui.allocate_exact_size(Vec2::new(260.0, 180.0), egui::Sense::hover());
        let painter = ui.painter();

        painter.rect_filled(rect, 4.0, Color32::from_rgb(14, 18, 25));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 75)), egui::StrokeKind::Inside);

        painter.text(Pos2::new(rect.left() + 8.0, rect.top() + 6.0), egui::Align2::LEFT_TOP, "P_shg(t) Transient Buildup", egui::FontId::proportional(11.0), Color32::from_rgb(180, 200, 230));

        let pad_l = 30.0;
        let pad_r = 15.0;
        let pad_b = 24.0;
        let pad_t = 24.0;

        let plot_w = rect.width() - pad_l - pad_r;
        let plot_h = rect.height() - pad_t - pad_b;

        let max_p = self.steady_state.p_out_mw.max(1.0) * 1.2;
        let max_t = 100.0_f32;

        if self.transient_curve.len() >= 2 {
            let mut points = Vec::new();
            for pt in &self.transient_curve {
                let x = rect.left() + pad_l + ((pt.time_ns as f32 / max_t).clamp(0.0, 1.0)) * plot_w;
                let y = rect.bottom() - pad_b - ((pt.p_shg_mw / max_p).clamp(0.0, 1.0) as f32) * plot_h;
                points.push(Pos2::new(x, y));
            }

            for i in 0..points.len() - 1 {
                painter.line_segment([points[i], points[i + 1]], Stroke::new(2.0, Color32::from_rgb(60, 220, 120)));
            }
        }
    }

    fn render_spectral_purity_bar(&self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.set_width(530.0);
            ui.label(RichText::new("Harmonic Output Spectrum & Spurious Suppression:").strong().color(Color32::from_rgb(200, 220, 255)));

            for sp in &self.harmonic_spectrum {
                ui.horizontal(|ui| {
                    ui.label(format!("{:<26}:", sp.label));
                    ui.label(RichText::new(format!("{:>6.2} GHz", sp.freq_ghz)).color(Color32::from_rgb(140, 180, 240)));
                    ui.label(RichText::new(format!("{:>6.2} mW", sp.power_mw)).color(Color32::WHITE));

                    let col = if sp.order == 2 {
                        Color32::from_rgb(80, 230, 255)
                    } else if sp.relative_power_db <= -30.0 {
                        Color32::from_rgb(60, 200, 100)
                    } else {
                        Color32::YELLOW
                    };
                    ui.label(RichText::new(format!("{:>7.1} dB", sp.relative_power_db)).color(col).strong());
                });
            }
        });
    }

    // TAB 3: Backscattering-Immune Routing
    fn render_tab_routing(&mut self, ui: &mut Ui) {
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.set_width(260.0);
                ui.label(RichText::new("Routing & Corner Controls").strong().color(Color32::from_rgb(180, 210, 255)));

                ui.label("Corner Bend Geometry:");
                if ui.radio_value(&mut self.router_params.bend_angle, CornerBendAngle::Deg90, "90 deg Sharp Corner").clicked() {
                    changed = true;
                }
                if ui.radio_value(&mut self.router_params.bend_angle, CornerBendAngle::Deg120, "120 deg Smooth Corner").clicked() {
                    changed = true;
                }

                ui.separator();
                ui.label(RichText::new("Defect & Obstacle Injection:").strong());
                if ui.checkbox(&mut self.router_params.has_defect, "Insert Vacancy Defect / Obstacle").changed() {
                    changed = true;
                }

                ui.separator();
                ui.label(RichText::new("Scattering S-Parameters:").strong());
                ui.label(format!("T_corner |S_21|^2: {:.2}%", self.scattering_matrix.s21_transmission_linear * 100.0));
                ui.label(format!("Insertion Loss: {:.3} dB", self.scattering_matrix.insertion_loss_db));
                ui.label(format!("Return Loss |S_11|^2: {:.1} dB", self.scattering_matrix.s11_return_loss_db));
                ui.label(format!("Reflection: {:.3}%", self.scattering_matrix.s11_reflection_linear * 100.0));
                ui.label(format!("Defect Immunity: {:.1}%", self.scattering_matrix.defect_immunity_ratio * 100.0));
            });

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.group(|ui| {
                        ui.label(RichText::new("Corner Transmission").size(11.0).color(Color32::GRAY));
                        let t = self.scattering_matrix.s21_transmission_linear;
                        let col = if t >= 0.95 { Color32::from_rgb(60, 210, 110) } else { Color32::YELLOW };
                        ui.label(RichText::new(format!("{:.1}%", t * 100.0)).color(col).strong().size(15.0));
                        ui.label("Target: >= 95.0%");
                    });

                    ui.group(|ui| {
                        ui.label(RichText::new("Return Loss |S_11|^2").size(11.0).color(Color32::GRAY));
                        let s11 = self.scattering_matrix.s11_return_loss_db;
                        let col = if s11 <= -25.0 { Color32::from_rgb(60, 210, 110) } else { Color32::YELLOW };
                        ui.label(RichText::new(format!("{:.1} dB", s11)).color(col).strong().size(15.0));
                        ui.label("Target: <= -25.0 dB");
                    });

                    ui.group(|ui| {
                        ui.label(RichText::new("Defect Immunity").size(11.0).color(Color32::GRAY));
                        let ratio = self.scattering_matrix.defect_immunity_ratio;
                        let col = if ratio >= 0.95 { Color32::from_rgb(60, 210, 110) } else { Color32::YELLOW };
                        ui.label(RichText::new(format!("{:.1}%", ratio * 100.0)).color(col).strong().size(15.0));
                        ui.label("Target: >= 95.0% T_clean");
                    });
                });

                // Canvas rendering wave propagation traversing sharp 90-degree corner
                self.render_routing_canvas(ui);
            });
        });

        if changed {
            self.recompute();
        }
    }

    fn render_routing_canvas(&self, ui: &mut Ui) {
        let (rect, _resp) = ui.allocate_exact_size(Vec2::new(480.0, 360.0), egui::Sense::hover());
        let painter = ui.painter();

        painter.rect_filled(rect, 4.0, Color32::from_rgb(14, 18, 25));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 75)), egui::StrokeKind::Inside);

        let grid = &self.routing_field.intensity;
        if grid.is_empty() || grid[0].is_empty() {
            return;
        }

        let sy = grid.len();
        let sx = grid[0].len();
        let pad = 24.0;
        let cell_w = (rect.width() - 2.0 * pad) / (sx as f32);
        let cell_h = (rect.height() - 2.0 * pad) / (sy as f32);

        for (iy, row) in grid.iter().enumerate() {
            let y_pos = rect.top() + pad + (iy as f32) * cell_h;
            for (ix, &val) in row.iter().enumerate() {
                let x_pos = rect.left() + pad + (ix as f32) * cell_w;
                let norm = val.clamp(0.0, 1.0);
                let color = sample_colormap(norm, self.selected_colormap);

                let cell_rect = Rect::from_min_size(Pos2::new(x_pos, y_pos), Vec2::new(cell_w - 0.5, cell_h - 0.5));
                painter.rect_filled(cell_rect, 1.0, color);
            }
        }

        // Draw obstacle / vacancy defect if present
        if let Some((dx, dy)) = self.routing_field.defect_pos {
            let defect_x = rect.left() + pad + (dx as f32 + 0.5) * cell_w;
            let defect_y = rect.top() + pad + (dy as f32 + 0.5) * cell_h;
            painter.circle_filled(Pos2::new(defect_x, defect_y), cell_w * 1.5, Color32::from_rgb(240, 50, 50));
            painter.circle_stroke(Pos2::new(defect_x, defect_y), cell_w * 1.5, Stroke::new(1.5, Color32::WHITE));
            painter.text(Pos2::new(defect_x, defect_y - cell_w * 2.2), egui::Align2::CENTER_BOTTOM, "Vacancy Defect", egui::FontId::proportional(11.0), Color32::from_rgb(255, 120, 120));
        }

        // Label inputs and outputs
        painter.text(Pos2::new(rect.left() + pad + 20.0, rect.bottom() - 10.0), egui::Align2::LEFT_BOTTOM, "Input Port (omega_1 -> 2*omega_1)", egui::FontId::proportional(11.0), Color32::from_rgb(140, 210, 255));
        painter.text(Pos2::new(rect.right() - pad - 20.0, rect.top() + 10.0), egui::Align2::RIGHT_TOP, "Output Port 2 (Deflected 90 deg)", egui::FontId::proportional(11.0), Color32::from_rgb(140, 255, 180));
    }

    // TAB 4: Multi-Octave Beam Steering
    fn render_tab_beam_steering(&mut self, ui: &mut Ui) {
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.set_width(260.0);
                ui.label(RichText::new("Beam Steering Port Selector").strong().color(Color32::from_rgb(180, 210, 255)));

                ui.label("Target Acoustic Output Port:");
                if ui.radio_value(&mut self.router_params.target_port, RouterTargetPort::Port1Forward, "Port 1: Forward (0 deg)").clicked() {
                    changed = true;
                }
                if ui.radio_value(&mut self.router_params.target_port, RouterTargetPort::Port2Deflected, "Port 2: Deflected (90 deg)").clicked() {
                    changed = true;
                }
                if ui.radio_value(&mut self.router_params.target_port, RouterTargetPort::Port3Isolated, "Port 3: Reverse (180 deg)").clicked() {
                    changed = true;
                }

                ui.separator();
                ui.label(RichText::new("Target Port Isolation:").strong());
                let iso = self.scattering_matrix.cross_port_isolation_db;
                let col = if iso >= 25.0 { Color32::from_rgb(60, 210, 110) } else { Color32::YELLOW };
                ui.label(RichText::new(format!("{:.1} dB", iso)).color(col).strong().size(16.0));
                ui.label("Spec: >= 25.0 dB Isolation");

                ui.separator();
                ui.label(RichText::new("Port Transmissions:").strong());
                for p in &self.scattering_matrix.ports {
                    let mark = if p.is_target { "(ACTIVE)" } else { "(ISOLATED)" };
                    ui.label(format!("{}: {:.1} dB {}", p.name, p.power_db, mark));
                }
            });

            ui.vertical(|ui| {
                self.render_beam_steering_canvas(ui);
            });
        });

        if changed {
            self.recompute();
        }
    }

    fn render_beam_steering_canvas(&self, ui: &mut Ui) {
        let (rect, _resp) = ui.allocate_exact_size(Vec2::new(480.0, 360.0), egui::Sense::hover());
        let painter = ui.painter();

        painter.rect_filled(rect, 4.0, Color32::from_rgb(14, 18, 25));
        painter.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 75)), egui::StrokeKind::Inside);

        let center = rect.center();
        let radius = 130.0_f32;

        // Draw polar circles
        for r_frac in [0.25, 0.50, 0.75, 1.0] {
            let r = radius * r_frac;
            painter.circle_stroke(center, r, Stroke::new(1.0, Color32::from_rgb(35, 45, 65)));
        }

        // Central corner nanocavity node
        painter.circle_filled(center, 14.0, Color32::from_rgb(60, 140, 255));
        painter.text(center, egui::Align2::CENTER_CENTER, "SHG", egui::FontId::proportional(11.0), Color32::WHITE);

        // Render port beams
        for port in &self.scattering_matrix.ports {
            let angle_rad = (port.angle_deg as f32).to_radians();
            let dir = Vec2::new(angle_rad.cos(), -angle_rad.sin());
            let port_pos = center + dir * radius;

            let (beam_col, beam_w, label_col) = if port.is_target {
                (Color32::from_rgb(50, 220, 110), 4.0, Color32::from_rgb(120, 255, 150))
            } else {
                (Color32::from_rgb(70, 80, 100), 1.5, Color32::from_rgb(140, 150, 170))
            };

            painter.line_segment([center, port_pos], Stroke::new(beam_w, beam_col));
            painter.circle_filled(port_pos, 8.0, beam_col);

            let text_pos = port_pos + dir * 18.0;
            painter.text(
                text_pos,
                egui::Align2::CENTER_CENTER,
                format!("{}\n{:.1} dB", port.name, port.power_db),
                egui::FontId::proportional(11.0),
                label_col,
            );
        }

        painter.text(
            Pos2::new(rect.left() + 10.0, rect.top() + 8.0),
            egui::Align2::LEFT_TOP,
            format!("Multi-Port Beam Steering (Target: {}, Isolation: {:.1} dB)", self.router_params.target_port.label(), self.scattering_matrix.cross_port_isolation_db),
            egui::FontId::proportional(11.0),
            Color32::from_rgb(200, 220, 255),
        );
    }

    // TAB 5: Audit & Telemetry
    fn render_tab_audit(&mut self, ui: &mut Ui) {
        let (passed, total) = self.audit_score;
        let all_pass = passed == total;

        ui.horizontal(|ui| {
            let badge_bg = if all_pass {
                Color32::from_rgb(20, 80, 40)
            } else {
                Color32::from_rgb(90, 60, 20)
            };
            let badge_fg = if all_pass {
                Color32::from_rgb(80, 230, 120)
            } else {
                Color32::from_rgb(250, 180, 50)
            };

            ui.group(|ui| {
                ui.label(RichText::new(format!("AUDIT RESULT: {} / {} SPECIFICATIONS VERIFIED", passed, total)).color(badge_fg).strong().size(15.0));
                ui.label(RichText::new(if all_pass { "100% SPECIFICATION COMPLIANCE - ALL CRITERIA PASS" } else { "ATTENTION: SOME SPECIFICATIONS OUT OF TOLERANCE" }).color(badge_bg));
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Re-run Full Audit Verification").clicked() {
                    self.recompute();
                }
            });
        });
        ui.separator();

        egui::Grid::new("corner_doubler_audit_table")
            .striped(true)
            .min_col_width(80.0)
            .show(ui, |ui| {
                ui.label(RichText::new("#").strong());
                ui.label(RichText::new("Criterion").strong());
                ui.label(RichText::new("Specification").strong());
                ui.label(RichText::new("Observed Value").strong());
                ui.label(RichText::new("Status").strong());
                ui.label(RichText::new("Technical Notes").strong());
                ui.end_row();

                for (idx, c) in self.audit_criteria.iter().enumerate() {
                    ui.label(format!("{}", idx + 1));
                    ui.label(RichText::new(&c.criterion).strong());
                    ui.label(&c.specification);
                    ui.label(RichText::new(&c.observed_state).color(Color32::from_rgb(180, 210, 255)));

                    if c.is_passed {
                        ui.label(RichText::new("PASS").color(Color32::from_rgb(50, 210, 110)).strong());
                    } else {
                        ui.label(RichText::new("FAIL").color(Color32::LIGHT_RED).strong());
                    }

                    ui.label(RichText::new(&c.technical_notes).size(11.0).color(Color32::GRAY));
                    ui.end_row();
                }
            });
    }
}

/// Helper function to interpolate Magma or Turbo colormap.
fn sample_colormap(t: f64, colormap: DialogColormap) -> Color32 {
    let t = t.clamp(0.0, 1.0) as f32;
    match colormap {
        DialogColormap::Magma => {
            // Black/Purple -> Reddish Violet -> Orange -> Bright Yellow
            if t < 0.25 {
                let frac = t / 0.25;
                Color32::from_rgb((frac * 60.0) as u8, (frac * 15.0) as u8, (frac * 80.0) as u8)
            } else if t < 0.55 {
                let frac = (t - 0.25) / 0.30;
                Color32::from_rgb((60.0 + frac * 120.0) as u8, (15.0 + frac * 30.0) as u8, (80.0 + frac * 40.0) as u8)
            } else if t < 0.85 {
                let frac = (t - 0.55) / 0.30;
                Color32::from_rgb((180.0 + frac * 65.0) as u8, (45.0 + frac * 115.0) as u8, (120.0 - frac * 80.0) as u8)
            } else {
                let frac = (t - 0.85) / 0.15;
                Color32::from_rgb(255, (160.0 + frac * 95.0) as u8, (40.0 + frac * 180.0) as u8)
            }
        }
        DialogColormap::Turbo => {
            // Deep Blue -> Cyan -> Green -> Yellow -> Red
            if t < 0.25 {
                let frac = t / 0.25;
                Color32::from_rgb((40.0 * (1.0 - frac)) as u8, (frac * 180.0) as u8, 240)
            } else if t < 0.50 {
                let frac = (t - 0.25) / 0.25;
                Color32::from_rgb((frac * 100.0) as u8, (180.0 + frac * 55.0) as u8, ((1.0 - frac) * 240.0) as u8)
            } else if t < 0.75 {
                let frac = (t - 0.50) / 0.25;
                Color32::from_rgb((100.0 + frac * 155.0) as u8, 235, (frac * 20.0) as u8)
            } else {
                let frac = (t - 0.75) / 0.25;
                Color32::from_rgb(255, ((1.0 - frac) * 235.0) as u8, 20)
            }
        }
    }
}
