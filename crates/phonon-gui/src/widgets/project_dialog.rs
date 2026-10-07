#![deny(unsafe_code)]

//! Project Manager modal dialog for Phonon Studio.
//!
//! Provides interactive project catalog browsing, saving, opening, deletion,
//! and export/import across desktop and browser storage.

use egui::{vec2, Align2, Color32, Key, Pos2, Rect, RichText, Stroke};

/// Active mode of the Project Manager dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectDialogMode {
    Open,
    SaveAs,
    Manager,
}

/// Action outcome from interacting with the ProjectDialog.
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectDialogAction {
    None,
    Open(String),
    SaveAs(String),
    Delete(String),
    Export(String),
    Close,
}

/// Project Manager modal window.
#[derive(Debug, Clone)]
pub struct ProjectDialog {
    pub is_open: bool,
    pub mode: ProjectDialogMode,
    pub project_name_buffer: String,
    pub selected_project: Option<String>,
    pub status_message: Option<String>,
}

impl Default for ProjectDialog {
    fn default() -> Self {
        Self {
            is_open: false,
            mode: ProjectDialogMode::Manager,
            project_name_buffer: String::new(),
            selected_project: None,
            status_message: None,
        }
    }
}

impl ProjectDialog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens the dialog in Open mode.
    pub fn open_for_open(&mut self) {
        self.mode = ProjectDialogMode::Open;
        self.status_message = None;
        self.is_open = true;
    }

    /// Opens the dialog in SaveAs mode with initial title suggestion.
    pub fn open_for_save_as(&mut self, current_title: &str) {
        self.mode = ProjectDialogMode::SaveAs;
        self.project_name_buffer = current_title.to_string();
        self.status_message = None;
        self.is_open = true;
    }

    /// Opens the dialog in general Manager mode.
    pub fn open_for_manager(&mut self) {
        self.mode = ProjectDialogMode::Manager;
        self.status_message = None;
        self.is_open = true;
    }

    /// Closes the dialog.
    pub fn close(&mut self) {
        self.is_open = false;
        self.status_message = None;
    }

    /// Renders the Project Manager dialog window.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        project_list: &[String],
    ) -> ProjectDialogAction {
        if !self.is_open {
            return ProjectDialogAction::None;
        }

        let mut action = ProjectDialogAction::None;

        // Render backdrop
        let screen_rect = ctx
            .input(|i| i.viewport().inner_rect)
            .unwrap_or(Rect::from_min_size(Pos2::ZERO, vec2(1920.0, 1080.0)));
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("project_dialog_backdrop"),
        ));
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(160));

        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.close();
            return ProjectDialogAction::Close;
        }

        let title = match self.mode {
            ProjectDialogMode::Open => "Open Project",
            ProjectDialogMode::SaveAs => "Save Project As",
            ProjectDialogMode::Manager => "Project Manager",
        };

        egui::Window::new(title)
            .collapsible(false)
            .resizable(true)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .default_size(vec2(520.0, 380.0))
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(18, 22, 30))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(56, 189, 248)))
                    .corner_radius(8.0)
                    .inner_margin(16.0),
            )
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Header description
                    let header_desc = match self.mode {
                        ProjectDialogMode::Open => "Select a saved project from storage to open:",
                        ProjectDialogMode::SaveAs => "Enter a name to save the current schematic project:",
                        ProjectDialogMode::Manager => "Manage saved schematic projects in local storage:",
                    };
                    ui.label(
                        RichText::new(header_desc)
                            .size(13.0)
                            .color(Color32::from_rgb(200, 210, 225)),
                    );

                    ui.add_space(8.0);

                    // Name input row for SaveAs mode
                    if self.mode == ProjectDialogMode::SaveAs {
                        ui.horizontal(|ui| {
                            ui.label("Project Name / Path:");
                            ui.text_edit_singleline(&mut self.project_name_buffer);
                        });
                        ui.add_space(4.0);
                        let target_display = if self.project_name_buffer.trim().is_empty() {
                            "Untitled.phn".to_string()
                        } else if self.project_name_buffer.ends_with(".phn") {
                            self.project_name_buffer.trim().to_string()
                        } else {
                            format!("{}.phn", self.project_name_buffer.trim())
                        };
                        ui.label(
                            RichText::new(format!("Destination: {}", target_display))
                                .size(11.0)
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                        ui.add_space(8.0);
                    }

                    // Project catalog list box
                    ui.label(RichText::new("Saved Projects:").strong().size(12.0));
                    egui::ScrollArea::vertical()
                        .max_height(180.0)
                        .show(ui, |ui| {
                            egui::Frame::new()
                                .fill(Color32::from_rgb(12, 16, 22))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(40, 50, 65)))
                                .corner_radius(4.0)
                                .inner_margin(6.0)
                                .show(ui, |ui| {
                                    if project_list.is_empty() {
                                        ui.label(
                                            RichText::new("No saved projects found.")
                                                .italics()
                                                .color(Color32::from_rgb(120, 130, 145)),
                                        );
                                    } else {
                                        for proj_name in project_list {
                                            let is_selected = self.selected_project.as_deref()
                                                == Some(proj_name.as_str());
                                            let response = ui.selectable_label(
                                                is_selected,
                                                RichText::new(proj_name)
                                                    .size(13.0)
                                                    .color(if is_selected {
                                                        Color32::from_rgb(56, 189, 248)
                                                    } else {
                                                        Color32::from_rgb(220, 230, 242)
                                                    }),
                                            );
                                            if response.clicked() {
                                                self.selected_project = Some(proj_name.clone());
                                                if self.mode == ProjectDialogMode::SaveAs {
                                                    self.project_name_buffer = proj_name.clone();
                                                }
                                            }
                                            if response.double_clicked() {
                                                self.selected_project = Some(proj_name.clone());
                                                if self.mode == ProjectDialogMode::Open {
                                                    action = ProjectDialogAction::Open(proj_name.clone());
                                                    self.close();
                                                }
                                            }
                                        }
                                    }
                                });
                        });

                    if let Some(msg) = &self.status_message {
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(msg)
                                .size(11.0)
                                .color(Color32::from_rgb(250, 204, 21)),
                        );
                    }

                    ui.add_space(14.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Action buttons
                    ui.horizontal(|ui| {
                        // Cancel / Close
                        if ui.button("Close").clicked() {
                            self.close();
                            action = ProjectDialogAction::Close;
                        }

                        // Delete button (Manager / Open mode)
                        if let Some(sel) = self.selected_project.clone() {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Delete")
                                            .color(Color32::from_rgb(248, 113, 113)),
                                    )
                                    .fill(Color32::from_rgb(40, 20, 25)),
                                )
                                .clicked()
                            {
                                action = ProjectDialogAction::Delete(sel.clone());
                                self.selected_project = None;
                            }

                            if ui.button("Export (.phn)").clicked() {
                                action = ProjectDialogAction::Export(sel);
                            }
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            match self.mode {
                                ProjectDialogMode::Open => {
                                    let can_open = self.selected_project.is_some();
                                    if ui
                                        .add_enabled(
                                            can_open,
                                            egui::Button::new(
                                                RichText::new("Open Project")
                                                    .strong()
                                                    .color(Color32::from_rgb(10, 15, 25)),
                                            )
                                            .fill(Color32::from_rgb(56, 189, 248)),
                                        )
                                        .clicked()
                                    {
                                        if let Some(sel) = &self.selected_project {
                                            action = ProjectDialogAction::Open(sel.clone());
                                            self.close();
                                        }
                                    }
                                }
                                ProjectDialogMode::SaveAs => {
                                    let can_save = !self.project_name_buffer.trim().is_empty();
                                    if ui
                                        .add_enabled(
                                            can_save,
                                            egui::Button::new(
                                                RichText::new("Save Project")
                                                    .strong()
                                                    .color(Color32::from_rgb(10, 15, 25)),
                                            )
                                            .fill(Color32::from_rgb(56, 189, 248)),
                                        )
                                        .clicked()
                                    {
                                        let name = self.project_name_buffer.trim().to_string();
                                        action = ProjectDialogAction::SaveAs(name);
                                        self.close();
                                    }
                                }
                                ProjectDialogMode::Manager => {
                                    if let Some(sel) = &self.selected_project {
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    RichText::new("Load Selected")
                                                        .strong()
                                                        .color(Color32::from_rgb(10, 15, 25)),
                                                )
                                                .fill(Color32::from_rgb(56, 189, 248)),
                                            )
                                            .clicked()
                                        {
                                            action = ProjectDialogAction::Open(sel.clone());
                                            self.close();
                                        }
                                    }
                                }
                            }
                        });
                    });
                });
            });

        action
    }
}
