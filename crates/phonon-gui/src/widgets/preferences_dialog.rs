#![deny(unsafe_code)]

//! Categorized Preferences modal dialog for Phonon Visual Studio.
//!
//! Provides categorized tabs:
//! - General (autosave, project prefix, auto-center)
//! - Canvas & Layout (grid mode, spacing, snap, toolbar button visibility)
//! - Theme & Colors (preset selection, color swatches, YAML import/export)
//! - Keybindings (exhaustive action registry list, search filter, shortcut assignment/reset)
//! - History & Undo (in-memory and on-disk depth limits)
//! - Thermal & Physics (ambient temperature, colormap selector, overlay toggle)

use crate::actions::{ActionCategory, ActionId, ActionRegistry};
use crate::preferences::AppPreferences;
use crate::schematic::HistoryStack;
use crate::thermal::Colormap;
use crate::theme::{PhononTheme, ThemePreset};
use egui::{Color32, Context, RichText, ScrollArea, Vec2, Window};

/// Active navigation category tab in the Preferences modal dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferencesTab {
    General,
    CanvasLayout,
    ThemeColors,
    Keybindings,
    HistoryUndo,
    ThermalPhysics,
}

impl PreferencesTab {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::General => "General",
            Self::CanvasLayout => "Canvas & Layout",
            Self::ThemeColors => "Theme & Colors",
            Self::Keybindings => "Keybindings",
            Self::HistoryUndo => "History & Undo",
            Self::ThermalPhysics => "Thermal & Physics",
        }
    }
}

/// Preferences modal dialog state and UI renderer.
#[derive(Debug, Clone)]
pub struct PreferencesDialog {
    pub is_open: bool,
    pub active_tab: PreferencesTab,
    pub keybind_search: String,
    pub editing_action: Option<ActionId>,
    pub shortcut_input_buffer: String,
    pub yaml_import_buffer: String,
    pub show_yaml_import_modal: bool,
    pub yaml_export_buffer: String,
    pub show_yaml_export_modal: bool,
    pub status_message: Option<String>,
}

impl Default for PreferencesDialog {
    fn default() -> Self {
        Self {
            is_open: false,
            active_tab: PreferencesTab::General,
            keybind_search: String::new(),
            editing_action: None,
            shortcut_input_buffer: String::new(),
            yaml_import_buffer: String::new(),
            show_yaml_import_modal: false,
            yaml_export_buffer: String::new(),
            show_yaml_export_modal: false,
            status_message: None,
        }
    }
}

impl PreferencesDialog {
    /// Creates a new `PreferencesDialog` instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens the dialog, optionally switching to a specific tab.
    pub fn open(&mut self, tab: Option<PreferencesTab>) {
        self.is_open = true;
        if let Some(t) = tab {
            self.active_tab = t;
        }
        self.status_message = None;
    }

    /// Closes the dialog.
    pub fn close(&mut self) {
        self.is_open = false;
        self.editing_action = None;
        self.show_yaml_import_modal = false;
        self.show_yaml_export_modal = false;
    }

    /// Renders the Preferences modal dialog. Returns `true` if any preference was modified.
    pub fn show(
        &mut self,
        ctx: &Context,
        preferences: &mut AppPreferences,
        registry: &ActionRegistry,
        history: &mut HistoryStack,
        thermal_colormap: &mut Colormap,
    ) -> bool {
        if !self.is_open {
            return false;
        }

        let mut changed = false;
        let mut open_flag = self.is_open;

        Window::new("Preferences")
            .open(&mut open_flag)
            .collapsible(false)
            .resizable(true)
            .default_size(Vec2::new(720.0, 520.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Left Sidebar Navigation
                    let nav_height = (ui.available_height() - 40.0).max(10.0);
                    ui.allocate_ui_with_layout(
                        Vec2::new(170.0, nav_height),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            ui.add_space(4.0);
                            let tabs = [
                                PreferencesTab::General,
                                PreferencesTab::CanvasLayout,
                                PreferencesTab::ThemeColors,
                                PreferencesTab::Keybindings,
                                PreferencesTab::HistoryUndo,
                                PreferencesTab::ThermalPhysics,
                            ];
                            for tab in tabs {
                                let is_selected = self.active_tab == tab;
                                let text = RichText::new(tab.display_name())
                                    .size(13.0)
                                    .color(if is_selected {
                                        Color32::from_rgb(100, 200, 255)
                                    } else {
                                        Color32::from_rgb(180, 190, 205)
                                    });
                                if ui
                                    .selectable_label(is_selected, text)
                                    .clicked()
                                {
                                    self.active_tab = tab;
                                    self.status_message = None;
                                }
                            }
                        },
                    );

                    ui.separator();

                    // Main Content Panel
                    let content_height = (ui.available_height() - 40.0).max(10.0);
                    ScrollArea::vertical()
                        .max_height(content_height)
                        .show(ui, |ui| {
                            ui.add_space(4.0);
                            match self.active_tab {
                                PreferencesTab::General => {
                                    changed |= Self::render_general_tab(ui, preferences);
                                }
                                PreferencesTab::CanvasLayout => {
                                    changed |= Self::render_canvas_tab(ui, preferences);
                                }
                                PreferencesTab::ThemeColors => {
                                    changed |= self.render_theme_tab(ui, preferences);
                                }
                                PreferencesTab::Keybindings => {
                                    changed |= self.render_keybindings_tab(ui, preferences, registry);
                                }
                                PreferencesTab::HistoryUndo => {
                                    changed |= Self::render_history_tab(ui, preferences, history);
                                }
                                PreferencesTab::ThermalPhysics => {
                                    changed |= Self::render_thermal_tab(ui, preferences, thermal_colormap);
                                }
                            }
                        });
                });

                ui.separator();

                // Bottom Action Bar
                ui.horizontal(|ui| {
                    if ui.button("Reset All to Defaults").clicked() {
                        *preferences = AppPreferences::default();
                        history.max_depth = preferences.in_memory_history_limit;
                        *thermal_colormap = Colormap::Turbo;
                        self.status_message = Some("All settings restored to default.".to_string());
                        changed = true;
                    }

                    if let Some(msg) = &self.status_message {
                        ui.label(RichText::new(msg).color(Color32::from_rgb(100, 220, 140)));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Close").clicked() {
                            self.close();
                        }
                    });
                });
            });

        self.is_open = open_flag;

        // Sub-modals for YAML Import/Export
        if self.show_yaml_import_modal {
            Window::new("Import Theme YAML")
                .collapsible(false)
                .resizable(true)
                .default_size(Vec2::new(500.0, 380.0))
                .show(ctx, |ui| {
                    ui.label("Paste YAML theme specification below:");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.yaml_import_buffer)
                            .desired_rows(14)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );
                    ui.horizontal(|ui| {
                        if ui.button("Apply Theme").clicked() {
                            match PhononTheme::from_yaml_str(&self.yaml_import_buffer) {
                                Ok(theme) => {
                                    preferences.custom_theme = Some(theme);
                                    self.show_yaml_import_modal = false;
                                    self.status_message = Some("Custom theme imported successfully.".to_string());
                                    changed = true;
                                }
                                Err(err) => {
                                    self.status_message = Some(format!("YAML parse error: {}", err));
                                }
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_yaml_import_modal = false;
                        }
                    });
                });
        }

        if self.show_yaml_export_modal {
            Window::new("Export Theme YAML")
                .collapsible(false)
                .resizable(true)
                .default_size(Vec2::new(500.0, 380.0))
                .show(ctx, |ui| {
                    ui.label("Theme YAML definition (copy to clipboard):");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.yaml_export_buffer)
                            .desired_rows(14)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );
                    ui.horizontal(|ui| {
                        if ui.button("Copy to Clipboard").clicked() {
                            ui.ctx().copy_text(self.yaml_export_buffer.clone());
                            self.status_message = Some("Theme YAML copied to clipboard.".to_string());
                        }
                        if ui.button("Done").clicked() {
                            self.show_yaml_export_modal = false;
                        }
                    });
                });
        }

        if changed {
            preferences.clamp_limits();
        }

        changed
    }

    fn render_general_tab(ui: &mut egui::Ui, prefs: &mut AppPreferences) -> bool {
        let mut changed = false;
        ui.heading("General Preferences");
        ui.add_space(8.0);

        ui.label(RichText::new("Session & Autosave").strong());
        if ui.checkbox(&mut prefs.autosave_enabled, "Enable periodic session autosave").changed() {
            changed = true;
        }

        ui.horizontal(|ui| {
            ui.label("Autosave interval (seconds):");
            if ui.add(egui::DragValue::new(&mut prefs.autosave_interval_secs).range(5..=600).speed(1.0)).changed() {
                changed = true;
            }
        });

        ui.add_space(8.0);
        ui.label(RichText::new("Project Defaults").strong());
        ui.horizontal(|ui| {
            ui.label("Default project name prefix:");
            if ui.text_edit_singleline(&mut prefs.default_project_prefix).changed() {
                changed = true;
            }
        });

        if ui.checkbox(&mut prefs.auto_center_on_load, "Auto-center schematic canvas on project load").changed() {
            changed = true;
        }

        changed
    }

    fn render_canvas_tab(ui: &mut egui::Ui, prefs: &mut AppPreferences) -> bool {
        let mut changed = false;
        ui.heading("Canvas & Layout Settings");
        ui.add_space(8.0);

        ui.label(RichText::new("Grid & Snapping").strong());
        if ui.checkbox(&mut prefs.show_grid, "Show canvas grid").changed() {
            changed = true;
        }

        ui.horizontal(|ui| {
            ui.label("Grid spacing (pixels):");
            if ui.add(egui::Slider::new(&mut prefs.grid_size, 10.0..=50.0).step_by(5.0)).changed() {
                changed = true;
            }
        });

        if ui.checkbox(&mut prefs.snap_to_grid, "Snap components and wires to grid").changed() {
            changed = true;
        }

        ui.add_space(12.0);
        ui.label(RichText::new("Action Toolbar Button Visibility").strong());
        ui.label("Customize visible quick-action shortcuts in the top toolbar:");

        if ui.checkbox(&mut prefs.show_toolbar_run_dc, "Show 'Run DC (.OP)' button").changed() {
            changed = true;
        }
        if ui.checkbox(&mut prefs.show_toolbar_run_transient, "Show 'Run Transient (.TRAN)' button").changed() {
            changed = true;
        }
        if ui.checkbox(&mut prefs.show_toolbar_export_netlist, "Show 'Export Netlist' button").changed() {
            changed = true;
        }
        if ui.checkbox(&mut prefs.show_toolbar_clear_canvas, "Show 'Clear Canvas' button").changed() {
            changed = true;
        }

        changed
    }

    fn render_theme_tab(&mut self, ui: &mut egui::Ui, prefs: &mut AppPreferences) -> bool {
        let mut changed = false;
        ui.heading("Theme & Visual Color Scheme");
        ui.add_space(8.0);

        ui.label(RichText::new("Preset Selection").strong());
        ui.horizontal(|ui| {
            ui.label("Theme Preset:");
            let cur_name = prefs.active_theme_preset.name();
            egui::ComboBox::from_id_salt("theme_preset_combo")
                .selected_text(cur_name)
                .show_ui(ui, |ui| {
                    for preset in ThemePreset::all() {
                        let is_sel = prefs.active_theme_preset == *preset && prefs.custom_theme.is_none();
                        if ui.selectable_label(is_sel, preset.name()).clicked() {
                            prefs.active_theme_preset = *preset;
                            prefs.custom_theme = None;
                            changed = true;
                        }
                    }
                });
        });

        if prefs.custom_theme.is_some() {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Custom theme active.").color(Color32::from_rgb(255, 200, 80)));
                if ui.button("Reset to Preset Defaults").clicked() {
                    prefs.custom_theme = None;
                    changed = true;
                }
            });
        }

        ui.add_space(10.0);
        ui.label(RichText::new("Theme Color Swatches").strong());

        let mut theme = prefs.current_theme();

        let swatch = |ui: &mut egui::Ui, label: &str, color: &mut Color32| -> bool {
            let mut c_changed = false;
            ui.horizontal(|ui| {
                ui.allocate_ui(Vec2::new(160.0, 20.0), |ui| {
                    ui.label(label);
                });
                if ui.color_edit_button_srgba(color).changed() {
                    c_changed = true;
                }
            });
            c_changed
        };

        let mut theme_edited = false;
        theme_edited |= swatch(ui, "Canvas Background:", &mut theme.canvas_bg);
        theme_edited |= swatch(ui, "Grid Dot:", &mut theme.grid_dot);
        theme_edited |= swatch(ui, "Grid Line:", &mut theme.grid_line);
        theme_edited |= swatch(ui, "Wire (Normal):", &mut theme.wire_normal);
        theme_edited |= swatch(ui, "Wire (Selected):", &mut theme.wire_selected);
        theme_edited |= swatch(ui, "Component Body:", &mut theme.component_body);
        theme_edited |= swatch(ui, "Component Stroke:", &mut theme.component_stroke);
        theme_edited |= swatch(ui, "Component Selected:", &mut theme.component_selected);
        theme_edited |= swatch(ui, "Pin (Normal):", &mut theme.pin_normal);
        theme_edited |= swatch(ui, "Pin (Connected):", &mut theme.pin_connected);
        theme_edited |= swatch(ui, "Accent Primary:", &mut theme.accent_primary);
        theme_edited |= swatch(ui, "Text Primary:", &mut theme.text_primary);
        theme_edited |= swatch(ui, "Text Secondary:", &mut theme.text_secondary);
        theme_edited |= swatch(ui, "Voltage Badge:", &mut theme.voltage_badge);

        if theme_edited {
            theme.name = "Custom Theme".to_string();
            prefs.custom_theme = Some(theme.clone());
            changed = true;
        }

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.button("Export Theme as YAML...").clicked() {
                self.yaml_export_buffer = theme.to_yaml_string();
                self.show_yaml_export_modal = true;
            }
            if ui.button("Import Theme from YAML...").clicked() {
                self.yaml_import_buffer.clear();
                self.show_yaml_import_modal = true;
            }
        });

        changed
    }

    fn render_keybindings_tab(
        &mut self,
        ui: &mut egui::Ui,
        prefs: &mut AppPreferences,
        registry: &ActionRegistry,
    ) -> bool {
        let mut changed = false;
        ui.heading("Keybindings & Shortcuts");
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("Search actions:");
            ui.text_edit_singleline(&mut self.keybind_search);
            if !self.keybind_search.is_empty() && ui.button("Clear").clicked() {
                self.keybind_search.clear();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Reset All Keybindings").clicked() {
                    prefs.reset_all_shortcuts();
                    self.editing_action = None;
                    self.status_message = Some("All keybindings reset to defaults.".to_string());
                    changed = true;
                }
            });
        });

        ui.add_space(6.0);

        let query = self.keybind_search.trim().to_lowercase();

        egui::Grid::new("keybinds_grid")
            .striped(true)
            .min_col_width(80.0)
            .show(ui, |ui| {
                ui.label(RichText::new("Action").strong());
                ui.label(RichText::new("Category").strong());
                ui.label(RichText::new("Default").strong());
                ui.label(RichText::new("Assigned").strong());
                ui.label(RichText::new("Controls").strong());
                ui.end_row();

                for action in registry.all() {
                    let title = action.title;
                    let desc = action.description;
                    let cat = match action.category {
                        ActionCategory::File => "File",
                        ActionCategory::Edit => "Edit",
                        ActionCategory::View => "View",
                        ActionCategory::Simulate => "Simulate",
                        ActionCategory::Tools => "Tools",
                    };
                    let def_sc = action.shortcut.unwrap_or("None");

                    // Filter
                    if !query.is_empty() {
                        let match_title = title.to_lowercase().contains(&query);
                        let match_desc = desc.to_lowercase().contains(&query);
                        let match_cat = cat.to_lowercase().contains(&query);
                        if !match_title && !match_desc && !match_cat {
                            continue;
                        }
                    }

                    let cur_sc_opt = prefs.get_shortcut(action.id, action.shortcut);
                    let cur_sc_display = cur_sc_opt.as_deref().unwrap_or("None");
                    let is_overridden = prefs.keybind_overrides.contains_key(&format!("{:?}", action.id));

                    ui.label(title);
                    ui.label(cat);
                    ui.monospace(def_sc);

                    if self.editing_action == Some(action.id) {
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut self.shortcut_input_buffer);
                            if ui.button("Assign").clicked() {
                                let new_val = if self.shortcut_input_buffer.trim().is_empty() {
                                    None
                                } else {
                                    Some(self.shortcut_input_buffer.trim().to_string())
                                };
                                prefs.set_shortcut(action.id, new_val);
                                self.editing_action = None;
                                changed = true;
                            }
                            if ui.button("Cancel").clicked() {
                                self.editing_action = None;
                            }
                        });
                    } else {
                        let text_color = if is_overridden {
                            Color32::from_rgb(255, 180, 50)
                        } else {
                            Color32::from_rgb(100, 200, 255)
                        };
                        ui.label(RichText::new(cur_sc_display).color(text_color).monospace());
                    }

                    ui.horizontal(|ui| {
                        if self.editing_action != Some(action.id) {
                            if ui.button("Edit").clicked() {
                                self.editing_action = Some(action.id);
                                self.shortcut_input_buffer = cur_sc_opt.clone().unwrap_or_default();
                            }
                            if is_overridden && ui.button("Reset").clicked() {
                                prefs.reset_shortcut(action.id);
                                changed = true;
                            }
                            if cur_sc_opt.is_some() && ui.button("Clear").clicked() {
                                prefs.set_shortcut(action.id, None);
                                changed = true;
                            }
                        }
                    });

                    ui.end_row();
                }
            });

        changed
    }

    fn render_history_tab(
        ui: &mut egui::Ui,
        prefs: &mut AppPreferences,
        history: &mut HistoryStack,
    ) -> bool {
        let mut changed = false;
        ui.heading("History & Undo Limits");
        ui.add_space(8.0);

        ui.label(RichText::new("In-Memory Undo Stack Depth").strong());
        ui.label("Maximum number of reversible actions preserved in RAM during the active session:");
        if ui
            .add(
                egui::Slider::new(&mut prefs.in_memory_history_limit, 10..=5000)
                    .logarithmic(true)
                    .text("actions"),
            )
            .changed()
        {
            history.max_depth = prefs.in_memory_history_limit;
            changed = true;
        }

        ui.add_space(10.0);
        ui.label(RichText::new("On-Disk Tail-Truncated History Depth").strong());
        ui.label("Maximum number of historical actions saved into .phn binary files to maintain minimal storage footprint:");
        if ui
            .add(
                egui::Slider::new(&mut prefs.on_disk_history_limit, 0..=500)
                    .text("actions"),
            )
            .changed()
        {
            changed = true;
        }

        ui.add_space(12.0);
        ui.separator();
        ui.label(format!("Active undo operations count: {}", history.undo_count()));
        ui.label(format!("Active redo operations count: {}", history.redo_count()));
        ui.label(format!("History clean baseline index: {}", history.clean_index));

        changed
    }

    fn render_thermal_tab(
        ui: &mut egui::Ui,
        prefs: &mut AppPreferences,
        thermal_colormap: &mut Colormap,
    ) -> bool {
        let mut changed = false;
        ui.heading("Thermal & Physics Controls");
        ui.add_space(8.0);

        ui.label(RichText::new("Thermal Colormap Visualization").strong());
        ui.horizontal(|ui| {
            ui.label("Colormap palette:");
            let cur_cmap = format!("{:?}", thermal_colormap);
            egui::ComboBox::from_id_salt("pref_colormap_combo")
                .selected_text(cur_cmap)
                .show_ui(ui, |ui| {
                    if ui.selectable_value(thermal_colormap, Colormap::Turbo, "Turbo").clicked() {
                        prefs.thermal_colormap = "Turbo".to_string();
                        changed = true;
                    }
                    if ui.selectable_value(thermal_colormap, Colormap::Magma, "Magma").clicked() {
                        prefs.thermal_colormap = "Magma".to_string();
                        changed = true;
                    }
                    if ui.selectable_value(thermal_colormap, Colormap::Inferno, "Inferno").clicked() {
                        prefs.thermal_colormap = "Inferno".to_string();
                        changed = true;
                    }
                });
        });

        ui.add_space(10.0);
        ui.label(RichText::new("Ambient Temperature").strong());
        let celsius = prefs.thermal_ambient_temp_k - 273.15;
        ui.horizontal(|ui| {
            ui.label("Ambient T_amb:");
            if ui
                .add(
                    egui::Slider::new(&mut prefs.thermal_ambient_temp_k, 200.0..=450.0)
                        .suffix(" K"),
                )
                .changed()
            {
                changed = true;
            }
            ui.label(format!("({:.1} °C)", celsius));
        });

        ui.add_space(10.0);
        if ui
            .checkbox(
                &mut prefs.thermal_overlay_enabled,
                "Show real-time thermal badges on schematic components",
            )
            .changed()
        {
            changed = true;
        }

        changed
    }
}
