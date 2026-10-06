#![deny(unsafe_code)]

//! Bespoke custom window top frame and brand menu bar system for Phonon Studio.
//!
//! Provides cross-platform unified titlebar, master SVG icon emblem, brand menus,
//! draggable window chrome, and desktop/web divergence controls.

use crate::schematic::compile_schematic;
use crate::widgets::confirmation_modal::{DemoCircuitKind, PendingAction};
use crate::widgets::icon::render_phonon_icon;
use egui::{pos2, vec2, Color32, FontId, OpenUrl, Rect, RichText, Sense, Stroke, StrokeKind, Ui, ViewportCommand};

/// Canonical URL for downloading the native Phonon desktop application.
pub const PHONON_RELEASES_URL: &str = "https://github.com/aerovexhq/phonon/releases/latest";

/// User interaction or window management action dispatched by the top frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopFrameAction {
    Minimize,
    Maximize,
    Close,
    DownloadDesktopApp,
    SaveProject,
    OpenProject,
    None,
}

/// Runtime configuration and display metadata for the custom top frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopFrameConfig {
    pub is_web: bool,
    pub title: String,
    pub app_version: String,
    pub circuit_name: String,
    pub is_modified: bool,
}

impl Default for TopFrameConfig {
    fn default() -> Self {
        Self {
            is_web: cfg!(target_arch = "wasm32"),
            title: "Phonon Studio".to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            circuit_name: "Untitled1".to_string(),
            is_modified: false,
        }
    }
}

impl TopFrameConfig {
    /// Creates a new `TopFrameConfig` with explicit parameters.
    pub fn new(
        is_web: bool,
        title: impl Into<String>,
        app_version: impl Into<String>,
        circuit_name: impl Into<String>,
    ) -> Self {
        Self {
            is_web,
            title: title.into(),
            app_version: app_version.into(),
            circuit_name: circuit_name.into(),
            is_modified: false,
        }
    }

    /// Builder method setting the modified state.
    pub fn with_modified(mut self, is_modified: bool) -> Self {
        self.is_modified = is_modified;
        self
    }

    /// Returns the formatted circuit display title: `{project_title}{*}`.
    pub fn formatted_title(&self) -> String {
        let name = if self.circuit_name.is_empty() {
            "Untitled1"
        } else {
            &self.circuit_name
        };
        let dirty = if self.is_modified { "*" } else { "" };
        format!("{}{}", name, dirty)
    }

    /// Returns whether native desktop window controls (minimize, maximize, close) are enabled.
    #[inline]
    pub fn has_window_controls(&self) -> bool {
        !self.is_web
    }

    /// Returns whether the web release download action is enabled.
    #[inline]
    pub fn has_download_action(&self) -> bool {
        self.is_web
    }

    /// Returns the window manipulation button labels for desktop mode: `("_", "[ ]", "X")`.
    #[inline]
    pub fn desktop_control_labels() -> (&'static str, &'static str, &'static str) {
        ("_", "[ ]", "X")
    }

    /// Returns the high-visibility action label for web mode.
    #[inline]
    pub fn web_download_action_label() -> &'static str {
        "Download Desktop App"
    }

    /// Returns the releases download URL.
    #[inline]
    pub fn download_url() -> &'static str {
        PHONON_RELEASES_URL
    }

    /// Fast analytical action resolver for low-latency simulation loops and benchmarks.
    #[inline]
    pub fn evaluate_action(
        &self,
        minimize_clicked: bool,
        maximize_clicked: bool,
        close_clicked: bool,
        download_clicked: bool,
    ) -> TopFrameAction {
        if self.is_web {
            if download_clicked {
                TopFrameAction::DownloadDesktopApp
            } else {
                TopFrameAction::None
            }
        } else if close_clicked {
            TopFrameAction::Close
        } else if maximize_clicked {
            TopFrameAction::Maximize
        } else if minimize_clicked {
            TopFrameAction::Minimize
        } else {
            TopFrameAction::None
        }
    }
}

/// Standalone top frame renderer operating on arbitrary egui UI contexts.
pub fn render_top_frame(ui: &mut Ui, config: &TopFrameConfig) -> TopFrameAction {
    render_top_frame_internal(ui, config, None)
}

/// App-integrated top frame renderer dispatching menu commands directly to `PhononApp`.
pub fn render_top_frame_with_app(
    ui: &mut Ui,
    config: &TopFrameConfig,
    app: &mut crate::PhononApp,
) -> TopFrameAction {
    render_top_frame_internal(ui, config, Some(app))
}

/// Internal shared top frame rendering engine.
fn render_top_frame_internal(
    ui: &mut Ui,
    config: &TopFrameConfig,
    mut app: Option<&mut crate::PhononApp>,
) -> TopFrameAction {
    let mut action = TopFrameAction::None;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 0.0);

        // 1. Top-Left: Vector Burger Menu Button for Palette Toggle
        let (burger_rect, burger_resp) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::click());
        if burger_resp.hovered() {
            ui.painter().rect_filled(burger_rect, 3.0, Color32::from_rgb(30, 41, 59));
        }
        let burger_color = if burger_resp.hovered() {
            Color32::from_rgb(125, 211, 252)
        } else {
            Color32::from_rgb(56, 189, 248)
        };
        let b_stroke = Stroke::new(1.8, burger_color);
        let bx_start = burger_rect.min.x + 3.0;
        let bx_end = burger_rect.max.x - 3.0;
        ui.painter().line_segment([pos2(bx_start, burger_rect.min.y + 5.0), pos2(bx_end, burger_rect.min.y + 5.0)], b_stroke);
        ui.painter().line_segment([pos2(bx_start, burger_rect.min.y + 10.0), pos2(bx_end, burger_rect.min.y + 10.0)], b_stroke);
        ui.painter().line_segment([pos2(bx_start, burger_rect.min.y + 15.0), pos2(bx_end, burger_rect.min.y + 15.0)], b_stroke);
        if burger_resp
            .on_hover_text("Toggle Component Palette (Burger Menu)")
            .clicked()
        {
            if let Some(a) = app.as_deref_mut() {
                a.show_palette = !a.show_palette;
            }
        }

        // Vectorized Master SVG Icon (width = 20, height = 20)
        render_phonon_icon(ui, 20.0);

        // 2. Title / Brand Label
        ui.label(
            RichText::new(&config.title)
                .strong()
                .size(13.0)
                .color(Color32::from_rgb(240, 246, 252)),
        );

        ui.label(RichText::new("|").color(Color32::from_rgb(60, 70, 85)).size(11.0));

        // 3. Main Menu Bar
        ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
        ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::NONE;

        // File Menu
        ui.menu_button("File", |ui| {
            if ui.button("New Project (Ctrl+N)").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.request_action(PendingAction::NewProject);
                }
                ui.close();
            }
            if ui.button("Open Project... (Ctrl+O)").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.open_open_dialog();
                } else {
                    action = TopFrameAction::OpenProject;
                }
                ui.close();
            }
            if ui.button("Save Project (Ctrl+S)").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    let _ = a.save_project();
                } else {
                    action = TopFrameAction::SaveProject;
                }
                ui.close();
            }
            if ui.button("Save Project As... (Ctrl+Shift+S)").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.open_save_as_dialog();
                }
                ui.close();
            }
            if ui.button("Rename Project...").clicked() {
                let cur = if config.circuit_name.is_empty() {
                    "Untitled1".to_string()
                } else {
                    config.circuit_name.clone()
                };
                ui.data_mut(|d| {
                    d.insert_temp(ui.id().with("top_frame_rename_active"), true);
                    d.insert_temp(ui.id().with("top_frame_rename_buffer"), cur);
                });
                ui.close();
            }
            ui.separator();
            ui.menu_button("Import", |ui| {
                if ui.button("SPICE Netlist (.cir, .net, .sp)...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.show_netlist_window = true;
                    }
                    ui.close();
                }
                if ui.button("Subcircuit Macro (.phnc)...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.subcircuit_dialog.is_open = true;
                    }
                    ui.close();
                }
            });
            ui.menu_button("Export", |ui| {
                if ui.button("SPICE Netlist (.cir)...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        if let Ok(compiled) = compile_schematic(&a.components, &a.wires) {
                            a.spice_netlist_text = compiled.spice_netlist;
                        }
                        a.show_netlist_window = true;
                    }
                    ui.close();
                }
                if ui.button("Phonon Project (.phn)...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        let _ = a.save_project();
                    }
                    ui.close();
                }
            });
            ui.separator();
            ui.menu_button("Load Demos", |ui| {
                if ui.button("Voltage Divider").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.request_action(PendingAction::LoadDemo(DemoCircuitKind::VoltageDivider));
                    }
                    ui.close();
                }
                if ui.button("Diode Clipper").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.request_action(PendingAction::LoadDemo(DemoCircuitKind::DiodeClipper));
                    }
                    ui.close();
                }
                if ui.button("BJT CE Amplifier").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.request_action(PendingAction::LoadDemo(DemoCircuitKind::BjtAmplifier));
                    }
                    ui.close();
                }
                if ui.button("CMOS Inverter").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.request_action(PendingAction::LoadDemo(DemoCircuitKind::CmosInverter));
                    }
                    ui.close();
                }
                if ui.button("NMOS Switch").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.request_action(PendingAction::LoadDemo(DemoCircuitKind::NmosSwitch));
                    }
                    ui.close();
                }
            });
            ui.separator();
            if ui.button("Preferences... (Ctrl+,)").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.preferences_dialog.is_open = true;
                }
                ui.close();
            }
            ui.separator();
            if ui.button("Exit (Alt+F4 / Ctrl+Q)").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.request_action(PendingAction::CloseApp);
                } else {
                    action = TopFrameAction::Close;
                }
                ui.close();
            }
        });
        ui.label(RichText::new("|").color(Color32::from_rgb(60, 70, 85)).size(11.0));

        // Edit Menu
        ui.menu_button("Edit", |ui| {
            let can_undo = app.as_ref().map_or(false, |a| a.history.can_undo());
            let can_redo = app.as_ref().map_or(false, |a| a.history.can_redo());

            if ui
                .add_enabled(can_undo, egui::Button::new("Undo (Ctrl+Z)"))
                .clicked()
            {
                if let Some(a) = app.as_deref_mut() {
                    a.undo();
                }
                ui.close();
            }
            if ui
                .add_enabled(can_redo, egui::Button::new("Redo (Ctrl+Y)"))
                .clicked()
            {
                if let Some(a) = app.as_deref_mut() {
                    a.redo();
                }
                ui.close();
            }
            ui.separator();
            if ui.button("Cut (Ctrl+X)").clicked() {
                ui.close();
            }
            if ui.button("Copy (Ctrl+C)").clicked() {
                ui.close();
            }
            if ui.button("Paste (Ctrl+V)").clicked() {
                ui.close();
            }
            if ui.button("Delete (Del)").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.delete_selected();
                }
                ui.close();
            }
            if ui.button("Duplicate (Ctrl+D)").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.duplicate_selected();
                }
                ui.close();
            }
            if ui.button("Select All (Ctrl+A)").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.select_all();
                }
                ui.close();
            }
            ui.separator();
            if ui.button("Clear Canvas...").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.request_action(PendingAction::ClearCanvas);
                }
                ui.close();
            }
            ui.separator();
            if ui.button("Command Palette (Ctrl+K)").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.command_palette.open();
                }
                ui.close();
            }
        });
        ui.label(RichText::new("|").color(Color32::from_rgb(60, 70, 85)).size(11.0));

        // View Menu
        ui.menu_button("View", |ui| {
            if let Some(a) = app.as_deref_mut() {
                ui.checkbox(&mut a.canvas.show_grid, "Show Grid (G)");
                ui.checkbox(&mut a.show_floating_toolbar, "Show Floating CAD Tools (H)");
                ui.checkbox(&mut a.show_oscilloscope, "Show Oscilloscope");
                ui.checkbox(&mut a.show_thermal_overlay, "Show Thermal Badges");
                ui.checkbox(&mut a.show_palette, "Show Component Palette");
                ui.checkbox(&mut a.show_erc_overlay, "Show ERC Overlay");
                ui.menu_button("Sheets", |ui| {
                    let mut switch_idx = None;
                    let mut add_new = false;
                    for (idx, sheet) in a.sheets.sheets.iter().enumerate() {
                        let is_active = idx == a.sheets.active_sheet_idx;
                        let prefix = if is_active { "* " } else { "  " };
                        if ui.button(format!("{}{}", prefix, sheet.name)).clicked() {
                            switch_idx = Some(idx);
                        }
                    }
                    ui.separator();
                    if ui.button("+ Add New Sheet").clicked() {
                        add_new = true;
                    }

                    if let Some(idx) = switch_idx {
                        a.switch_to_sheet(idx);
                        ui.close();
                    }
                    if add_new {
                        let count = a.sheets.sheets.len() + 1;
                        let new_name = format!("Sheet {}", count);
                        a.add_sheet(&new_name);
                        ui.close();
                    }
                });
                if ui.button("Center View (Auto-Fit)").clicked() {
                    a.pending_auto_center = true;
                    ui.close();
                }
                if ui.button("Reset View").clicked() {
                    a.canvas.pan = egui::Vec2::new(100.0, 100.0);
                    a.canvas.zoom = 1.0;
                    ui.close();
                }
            } else {
                let mut dummy_grid = true;
                let mut dummy_scope = true;
                let mut dummy_thermal = true;
                let mut dummy_erc = true;
                ui.checkbox(&mut dummy_grid, "Show Grid");
                ui.checkbox(&mut dummy_scope, "Show Oscilloscope");
                ui.checkbox(&mut dummy_thermal, "Show Thermal Badges");
                ui.checkbox(&mut dummy_erc, "Show ERC Overlay");
                if ui.button("Center View (Auto-Fit)").clicked() {
                    ui.close();
                }
                if ui.button("Reset View").clicked() {
                    ui.close();
                }
            }
        });
        ui.label(RichText::new("|").color(Color32::from_rgb(60, 70, 85)).size(11.0));

        // Simulate Menu
        ui.menu_button("Simulate", |ui| {
            if ui.button("Run DC Operating Point (.OP)").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.run_dc_op();
                }
                ui.close();
            }
            if ui.button("Run Transient (.TRAN)").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.run_transient_demo();
                }
                ui.close();
            }
            if ui.button("Run ERC Rules Check").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.run_erc();
                }
                ui.close();
            }
            ui.separator();
            if ui.button("Netlist Inspector...").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    if let Ok(compiled) = compile_schematic(&a.components, &a.wires) {
                        a.spice_netlist_text = compiled.spice_netlist;
                    }
                    a.show_netlist_window = true;
                }
                ui.close();
            }
            if ui.button("Clear Oscilloscope Traces").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.oscilloscope.clear();
                    a.multi_graph.primary_scope.clear();
                }
                ui.close();
            }
            ui.separator();
            if ui.button("Lua Testbench Console...").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.lua_console_dialog.is_open = true;
                }
                ui.close();
            }
        });
        ui.label(RichText::new("|").color(Color32::from_rgb(60, 70, 85)).size(11.0));

        // Analysis Menu
        ui.menu_button("Analysis", |ui| {
            if ui.button("Monte Carlo Yield Analysis...").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.monte_carlo_dialog.is_open = true;
                }
                ui.close();
            }
            if ui.button("Transient Sensitivity Analysis...").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.sensitivity_dialog.is_open = true;
                }
                ui.close();
            }
            if ui.button("Distributed Cluster Sweep...").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.cluster_dashboard_dialog.is_open = true;
                }
                ui.close();
            }
            ui.separator();
            if ui.button("RF S-Parameters & Smith Chart...").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.smith_chart_dialog.is_open = true;
                }
                ui.close();
            }
            if ui.button("Thermal Floorplan & Co-Sim...").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.thermal_floorplan_dialog.is_open = true;
                }
                ui.close();
            }
            if ui.button("Superconducting JTWPA Amplifier...").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.jtwpa_dialog.is_open = true;
                }
                ui.close();
            }
            ui.separator();
            ui.menu_button("Advanced Quantum & Metamaterials", |ui| {
                if ui.button("Polariton Waveguide & Cavity...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.polariton_cavity_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Neuromorphic SNN Studio...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.neuromorphic_snn_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Exceptional Point & PT Circuit...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.exceptional_point_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Weyl Semimetal Studio...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.weyl_semimetal_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Anyon Braiding & FQH Studio...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.fqh_braiding_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Floquet Acoustic Metasurface...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.floquet_metasurface_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Holonomic Quantum Processor...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.holonomic_processor_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Twisted Bilayer Moire Studio...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.twisted_moire_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Acoustic Chern Circulator...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.chern_circulator_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Kerr Microcomb Studio...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.kerr_microcomb_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Exceptional Surface Sensor Array...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.exceptional_surface_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Topological SOTI Corner Resonator...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.soti_corner_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Topological Lieb Lattice Flat-Band...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.lieb_lattice_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Higher-Order Axion Insulator...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.axion_insulator_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Floquet Time Crystal Studio...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.floquet_time_crystal_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Quantum Braiding Lattice Processor...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.quantum_braiding_lattice_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Cavity Optomechanical Squeezing...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.optomechanical_squeezing_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Acoustic Skyrmion Router...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.skyrmion_router_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Acoustic Domain Wall Solitons...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.acoustic_soliton_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Valley Acoustic Multiplexer...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.valley_multiplexer_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Non-Hermitian Skin Effect Sensor...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.non_hermitian_skin_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Topological Quadrupole SHG...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.quadrupole_shg_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Topological 4D QHE & Synthetic Dimensions...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.synthetic_4d_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("PT-Symmetric Acoustic Invisibility...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.pt_symmetric_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Topological Acoustic BIC & Vortex Cavity...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.acoustic_bic_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Non-Abelian Euler Class Metamaterial...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.euler_acoustic_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Topological Acoustic Octupole Insulator...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.octupole_insulator_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Topological Acoustic Moire Quasicrystal...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.aah_quasicrystal_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Acoustic Valley-Hall Vortex Pumping...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.valley_hall_vortex_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Acoustic Skyrmion Beam Deflector...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.skyrmion_deflector_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Floquet Synthetic Frequency Dimension...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.floquet_frequency_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Non-Hermitian Corner Laser...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.corner_laser_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Directional Heavy Ion Radiation...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.directional_radiation_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Atmospheric Neutron & DO-254 SER...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.atmospheric_neutron_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Thermal-Vacuum & Orbital Cycling...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.thermal_vacuum_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("SpaceWire/SpaceFibre & AFDX Bus...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.space_avionics_bus_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("RHBD DRC & Autonomous Self-Healing...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.rhbd_self_healing_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("2.5D/3D Chiplet Packaging...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.chiplet_packaging_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Dynamic DVFS & Thermal Throttling...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.electrothermal_throttling_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Power Delivery Network (PDN)...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.pdn_droop_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Silicon Aging & Reliability (EM/BTI)...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.silicon_aging_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Wafer-Scale Yield & DFM...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.wafer_yield_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Design Space Exploration (DSE)...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.dse_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Silicon Lifecycle & On-Die Telemetry...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.slm_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Microscopic Wavepacket Scattering...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.wavepacket_scattering_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Skyrmion Reservoir Computing...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.skyrmion_reservoir_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Phonon-Magnon Polariton Transducer...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.phonon_magnon_dialog.is_open = true;
                    }
                    ui.close();
                }
                if ui.button("Topological Quadrupole Parametric Waveguide...").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.quadrupole_parametric_dialog.is_open = true;
                    }
                    ui.close();
                }
            });
        });
        ui.label(RichText::new("|").color(Color32::from_rgb(60, 70, 85)).size(11.0));

        // Tools Menu
        ui.menu_button("Tools", |ui| {
            if ui.button("SPICE Model Extraction Wizard...").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.extraction_wizard.is_open = true;
                }
                ui.close();
            }
            if ui.button("Component Symbol & Shape Editor...").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.symbol_editor.is_open = true;
                }
                ui.close();
            }
            if ui.button("Command Palette (Ctrl+K)").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.command_palette.open();
                }
                ui.close();
            }
            if ui.button("Production Economics & BOM Cost Estimator...").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.production_economics_dialog.is_open = true;
                }
                ui.close();
            }
            if ui.button("WASM Deployment & Cache Profiler...").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.wasm_optimization_dialog.is_open = true;
                }
                ui.close();
            }
            if ui.button("Progressive Web App (PWA) & Offline Cache...").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.pwa_dialog.is_open = true;
                }
                ui.close();
            }
            if ui.button("Native Desktop Studio & IPC Co-Processor...").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.desktop_ipc_dialog.is_open = true;
                }
                ui.close();
            }
            if ui.button("Collaborative WebRTC Mesh...").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.webrtc_mesh_dialog.is_open = true;
                }
                ui.close();
            }
            if ui.button("WebGPU Compute Shader SPICE Co-Processor...").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.webgpu_spice_dialog.is_open = true;
                }
                ui.close();
            }
        });
        ui.label(RichText::new("|").color(Color32::from_rgb(60, 70, 85)).size(11.0));

        // Help Menu
        ui.menu_button("Help", |ui| {
            if ui.button("Documentation").clicked() {
                ui.ctx().open_url(OpenUrl::new_tab("https://github.com/aerovexhq/phonon#readme"));
                ui.close();
            }
            if ui.button("Keyboard Shortcuts").clicked() {
                ui.close();
            }
            if ui.button("About").clicked() {
                ui.close();
            }
        });

        // 4. Center Area: Draggable Title Bar with Circuit Name & Version
        let right_controls_width = if config.is_web { 165.0 } else { 92.0 };
        let available = ui.available_width();
        let center_width = (available - right_controls_width).max(20.0);

        let (center_rect, center_resp) =
            ui.allocate_exact_size(vec2(center_width, 22.0), Sense::click_and_drag());

        // Draggable window support
        if center_resp.drag_started_by(egui::PointerButton::Primary) {
            ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
        }
        if center_resp.double_clicked() {
            let is_max = ui.input(|i| i.viewport().maximized.unwrap_or(false));
            ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(!is_max));
        }

        let edit_id = ui.id().with("top_frame_rename_active");
        let edit_buf_id = ui.id().with("top_frame_rename_buffer");
        let mut is_renaming: bool = ui.data(|d| d.get_temp(edit_id).unwrap_or(false));

        if center_resp.secondary_clicked() {
            is_renaming = true;
            ui.data_mut(|d| {
                d.insert_temp(edit_id, true);
                let cur = if config.circuit_name.is_empty() {
                    "Untitled1".to_string()
                } else {
                    config.circuit_name.clone()
                };
                d.insert_temp(edit_buf_id, cur);
            });
        }

        if is_renaming {
            let mut buf: String = ui.data(|d| {
                d.get_temp(edit_buf_id).unwrap_or_else(|| {
                    if config.circuit_name.is_empty() {
                        "Untitled1".to_string()
                    } else {
                        config.circuit_name.clone()
                    }
                })
            });
            let edit_response = ui.put(
                center_rect,
                egui::TextEdit::singleline(&mut buf)
                    .font(FontId::monospace(11.0))
                    .hint_text("Project Title"),
            );

            ui.data_mut(|d| d.insert_temp(edit_buf_id, buf.clone()));

            if edit_response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                ui.data_mut(|d| d.insert_temp(edit_id, false));
                let trimmed = buf.trim();
                if !trimmed.is_empty() {
                    if let Some(a) = app.as_deref_mut() {
                        a.rename_project(trimmed);
                    }
                }
            }
        } else {
            // Display {project_title}{*} (showing * when modified)
            let circuit_display = config.formatted_title();
            ui.painter().text(
                center_rect.center(),
                egui::Align2::CENTER_CENTER,
                &circuit_display,
                FontId::monospace(11.0),
                Color32::from_rgb(148, 163, 184),
            );
        }

        // 5. Top-Right Window Controls
        if config.is_web {
            // Web Mode: High-visibility Download Desktop App action button
            let dl_btn = egui::Button::new(
                RichText::new("Download Desktop App")
                    .size(11.0)
                    .color(Color32::from_rgb(255, 255, 255))
                    .strong(),
            )
            .fill(Color32::from_rgb(14, 116, 144))
            .stroke(Stroke::new(1.0, Color32::from_rgb(56, 189, 248)));

            if ui.add(dl_btn).clicked() {
                ui.ctx().open_url(OpenUrl::new_tab(PHONON_RELEASES_URL));
                action = TopFrameAction::DownloadDesktopApp;
            }
        } else {
            // Desktop Mode: Modern Vector Window Controls (Minimize, Maximize/Restore, Close)
            let btn_size = vec2(26.0, 22.0);

            // 1. Minimize Vector Icon
            let (min_rect, min_resp) = ui.allocate_exact_size(btn_size, Sense::click());
            if min_resp.hovered() {
                ui.painter().rect_filled(min_rect, 2.0, Color32::from_rgb(30, 41, 59));
            }
            let min_color = if min_resp.hovered() {
                Color32::from_rgb(248, 250, 252)
            } else {
                Color32::from_rgb(148, 163, 184)
            };
            let min_c = min_rect.center();
            ui.painter().line_segment(
                [pos2(min_c.x - 4.5, min_c.y + 3.0), pos2(min_c.x + 4.5, min_c.y + 3.0)],
                Stroke::new(1.3, min_color),
            );
            if min_resp.clicked() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
                action = TopFrameAction::Minimize;
            }

            // 2. Maximize / Restore Vector Icon
            let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
            let (max_rect, max_resp) = ui.allocate_exact_size(btn_size, Sense::click());
            if max_resp.hovered() {
                ui.painter().rect_filled(max_rect, 2.0, Color32::from_rgb(30, 41, 59));
            }
            let max_color = if max_resp.hovered() {
                Color32::from_rgb(248, 250, 252)
            } else {
                Color32::from_rgb(148, 163, 184)
            };
            let max_c = max_rect.center();
            if is_maximized {
                // Restore icon: two overlapping boxes
                ui.painter().rect_stroke(
                    Rect::from_min_size(pos2(max_c.x - 2.5, max_c.y - 4.5), vec2(7.0, 7.0)),
                    1.0,
                    Stroke::new(1.1, max_color),
                    StrokeKind::Middle,
                );
                ui.painter().rect_stroke(
                    Rect::from_min_size(pos2(max_c.x - 4.5, max_c.y - 2.5), vec2(7.0, 7.0)),
                    1.0,
                    Stroke::new(1.1, max_color),
                    StrokeKind::Middle,
                );
            } else {
                // Maximize icon: single box
                ui.painter().rect_stroke(
                    Rect::from_center_size(max_c, vec2(8.5, 8.5)),
                    1.0,
                    Stroke::new(1.2, max_color),
                    StrokeKind::Middle,
                );
            }
            if max_resp.clicked() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
                action = TopFrameAction::Maximize;
            }

            // 3. Close Vector Icon
            let (close_rect, close_resp) = ui.allocate_exact_size(btn_size, Sense::click());
            if close_resp.hovered() {
                ui.painter().rect_filled(close_rect, 2.0, Color32::from_rgb(225, 29, 72));
            }
            let close_color = if close_resp.hovered() {
                Color32::from_rgb(255, 255, 255)
            } else {
                Color32::from_rgb(239, 68, 68)
            };
            let close_c = close_rect.center();
            let d = 3.5;
            ui.painter().line_segment(
                [pos2(close_c.x - d, close_c.y - d), pos2(close_c.x + d, close_c.y + d)],
                Stroke::new(1.3, close_color),
            );
            ui.painter().line_segment(
                [pos2(close_c.x - d, close_c.y + d), pos2(close_c.x + d, close_c.y - d)],
                Stroke::new(1.3, close_color),
            );
            if close_resp.clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.request_action(PendingAction::CloseApp);
                } else {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                }
                action = TopFrameAction::Close;
            }
        }
    });

    action
}
