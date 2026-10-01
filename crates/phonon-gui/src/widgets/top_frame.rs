#![deny(unsafe_code)]

//! Bespoke custom window top frame and brand menu bar system for Phonon Studio.
//!
//! Provides cross-platform unified titlebar, master SVG icon emblem, brand menus,
//! draggable window chrome, and desktop/web divergence controls.

use crate::schematic::compile_schematic;
use crate::widgets::icon::render_phonon_icon;
use egui::{vec2, Color32, FontId, OpenUrl, RichText, Sense, Stroke, Ui, ViewportCommand};

/// Canonical URL for downloading the native Phonon desktop application.
pub const PHONON_RELEASES_URL: &str = "https://github.com/aerovexsim/phonon/releases/latest";

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
}

impl Default for TopFrameConfig {
    fn default() -> Self {
        Self {
            is_web: cfg!(target_arch = "wasm32"),
            title: "Phonon Studio".to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            circuit_name: "Untitled Circuit".to_string(),
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
        }
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

        // 1. Top-Left: Vectorized Master SVG Icon (width = 20, height = 20)
        render_phonon_icon(ui, 20.0);

        // 2. Title / Brand Label
        ui.label(
            RichText::new(&config.title)
                .strong()
                .size(13.0)
                .color(Color32::from_rgb(240, 246, 252)),
        );

        ui.separator();

        // 3. Main Menu Bar
        // File Menu
        ui.menu_button("File", |ui| {
            if ui.button("New").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.clear_all();
                }
                ui.close();
            }
            if ui.button("Save Project (.phn)").clicked() {
                action = TopFrameAction::SaveProject;
                ui.close();
            }
            if ui.button("Open Project (.phn)").clicked() {
                action = TopFrameAction::OpenProject;
                ui.close();
            }
            ui.menu_button("Load Demos", |ui| {
                if ui.button("Voltage Divider").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.load_voltage_divider_demo();
                    }
                    ui.close();
                }
                if ui.button("Diode Clipper").clicked() {
                    if let Some(a) = app.as_deref_mut() {
                        a.load_diode_clipper_demo();
                    }
                    ui.close();
                }
            });
            if ui.button("Export SPICE Netlist").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    if let Ok(compiled) = compile_schematic(&a.components, &a.wires) {
                        a.spice_netlist_text = compiled.spice_netlist;
                    }
                    a.show_netlist_window = true;
                }
                ui.close();
            }
            ui.separator();
            if ui.button("Exit").clicked() {
                action = TopFrameAction::Close;
                ui.close();
            }
        });

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
            if ui.button("Select All (Ctrl+A)").clicked() {
                ui.close();
            }
        });

        // View Menu
        ui.menu_button("View", |ui| {
            if let Some(a) = app.as_deref_mut() {
                ui.checkbox(&mut a.canvas.show_grid, "Show Grid");
                ui.checkbox(&mut a.show_oscilloscope, "Show Oscilloscope");
                ui.checkbox(&mut a.show_thermal_overlay, "Show Thermal Badges");
                if ui.button("Toggle ERC Overlay").clicked() {
                    a.show_erc_overlay = !a.show_erc_overlay;
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
                ui.checkbox(&mut dummy_grid, "Show Grid");
                ui.checkbox(&mut dummy_scope, "Show Oscilloscope");
                ui.checkbox(&mut dummy_thermal, "Show Thermal Badges");
                if ui.button("Toggle ERC Overlay").clicked() {
                    ui.close();
                }
                if ui.button("Reset View").clicked() {
                    ui.close();
                }
            }
        });

        // Simulation Menu
        ui.menu_button("Simulation", |ui| {
            if ui.button("Run DC .OP").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.run_dc_op();
                }
                ui.close();
            }
            if ui.button("Run Transient .TRAN").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.run_transient_demo();
                }
                ui.close();
            }
            if ui.button("Run ERC Check").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.run_erc();
                }
                ui.close();
            }
            if ui.button("Clear Traces").clicked() {
                if let Some(a) = app.as_deref_mut() {
                    a.oscilloscope.clear();
                }
                ui.close();
            }
        });

        // Help Menu
        ui.menu_button("Help", |ui| {
            if ui.button("Documentation").clicked() {
                ui.ctx().open_url(OpenUrl::new_tab("https://github.com/aerovexsim/phonon#readme"));
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

        // Circuit name and status in center
        let circuit_display = if config.circuit_name.is_empty() {
            "Untitled Circuit".to_string()
        } else {
            format!("{} (v{})", config.circuit_name, config.app_version)
        };
        ui.painter().text(
            center_rect.center(),
            egui::Align2::CENTER_CENTER,
            circuit_display,
            FontId::monospace(11.0),
            Color32::from_rgb(148, 163, 184),
        );

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
            // Desktop Mode: Minimize, Maximize/Restore, Close buttons
            // Minimize button "_"
            let min_btn = egui::Button::new(RichText::new(" _ ").monospace().size(12.0));
            if ui.add(min_btn).clicked() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
                action = TopFrameAction::Minimize;
            }

            // Maximize/Restore button "[ ]"
            let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
            let max_label = if is_maximized { "[=]" } else { "[ ]" };
            let max_btn = egui::Button::new(RichText::new(max_label).monospace().size(11.0));
            if ui.add(max_btn).clicked() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
                action = TopFrameAction::Maximize;
            }

            // Close button "X"
            let close_btn = egui::Button::new(
                RichText::new(" X ")
                    .monospace()
                    .size(11.0)
                    .color(Color32::from_rgb(239, 68, 68)),
            );
            if ui.add(close_btn).clicked() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                action = TopFrameAction::Close;
            }
        }
    });

    action
}
