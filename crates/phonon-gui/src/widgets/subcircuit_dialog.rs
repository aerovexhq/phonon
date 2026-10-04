#![deny(unsafe_code)]

//! Reusable component packaging modal dialog for Phonon Studio.

use crate::schematic::components::SchematicComponent;
use crate::schematic::subcircuit_package::{
    discover_boundary_ports, PortDirection, PortEdge, SubcircuitPackage, SubcircuitPort,
};
use crate::schematic::wire::SchematicWire;
use egui::{Color32, Context, RichText, Window};

/// Action outcome from the subcircuit packaging dialog.
#[derive(Debug, Clone, PartialEq)]
pub enum SubcircuitDialogAction {
    SavePackage(SubcircuitPackage),
    Cancel,
}

/// Modal dialog for packaging selected components into a reusable `.phnc` subcircuit block.
#[derive(Debug, Clone)]
pub struct SubcircuitPackageDialog {
    pub is_open: bool,
    pub package_name: String,
    pub description: String,
    pub author: String,
    pub ports: Vec<SubcircuitPort>,
    pub internal_components: Vec<SchematicComponent>,
    pub internal_wires: Vec<SchematicWire>,
    pub show_spice_preview: bool,
    pub status_message: String,
}

impl Default for SubcircuitPackageDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl SubcircuitPackageDialog {
    pub fn new() -> Self {
        Self {
            is_open: false,
            package_name: "Subcircuit1".to_string(),
            description: "Custom reusable subcircuit block".to_string(),
            author: "Phonon User".to_string(),
            ports: Vec::new(),
            internal_components: Vec::new(),
            internal_wires: Vec::new(),
            show_spice_preview: false,
            status_message: String::new(),
        }
    }

    /// Opens the dialog initialized with the given components and wires.
    pub fn open_with_selection(
        &mut self,
        components: Vec<SchematicComponent>,
        wires: Vec<SchematicWire>,
    ) {
        self.ports = discover_boundary_ports(&components, &wires);
        self.internal_components = components;
        self.internal_wires = wires;
        self.package_name = "NewSubcircuit".to_string();
        self.description = "Reusable functional block".to_string();
        self.status_message.clear();
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.status_message.clear();
    }

    /// Renders the modal window and returns an action if submitted.
    pub fn show(&mut self, ctx: &Context) -> Option<SubcircuitDialogAction> {
        if !self.is_open {
            return None;
        }

        let mut action = None;
        let mut open = self.is_open;

        Window::new("Package Reusable Subcircuit (.phnc)")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(520.0)
            .show(ctx, |ui| {
                ui.heading("Register Component Globally");
                ui.label(
                    RichText::new("Bundle canvas components into a self-contained, reusable hierarchical block with boundary ports.")
                        .italics()
                        .color(Color32::from_rgb(160, 180, 200)),
                );
                ui.separator();

                egui::Grid::new("subckt_meta_grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Package Name:").strong());
                        ui.text_edit_singleline(&mut self.package_name);
                        ui.end_row();

                        ui.label(RichText::new("Description:").strong());
                        ui.text_edit_singleline(&mut self.description);
                        ui.end_row();

                        ui.label(RichText::new("Author:").strong());
                        ui.text_edit_singleline(&mut self.author);
                        ui.end_row();
                    });

                ui.add_space(8.0);
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "Boundary Ports ({}) [{} Components, {} Wires]",
                            self.ports.len(),
                            self.internal_components.len(),
                            self.internal_wires.len()
                        ))
                        .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("+ Add Port").clicked() {
                            let idx = self.ports.len() + 1;
                            self.ports.push(SubcircuitPort::new(
                                &format!("P{}", idx),
                                PortDirection::Bidirectional,
                                PortEdge::Left,
                                &format!("net{}", idx),
                            ));
                        }
                    });
                });

                ui.add_space(4.0);

                // Port editor table
                egui::ScrollArea::vertical()
                    .max_height(160.0)
                    .show(ui, |ui| {
                        let mut to_remove = None;
                        for (idx, port) in self.ports.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("#{}:", idx + 1)).monospace());
                                ui.add(
                                    egui::TextEdit::singleline(&mut port.name)
                                        .desired_width(70.0)
                                        .hint_text("Name"),
                                );

                                // Direction combo
                                egui::ComboBox::from_id_salt(format!("port_dir_{}", idx))
                                    .selected_text(port.direction.display_name())
                                    .width(90.0)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut port.direction,
                                            PortDirection::Input,
                                            "Input",
                                        );
                                        ui.selectable_value(
                                            &mut port.direction,
                                            PortDirection::Output,
                                            "Output",
                                        );
                                        ui.selectable_value(
                                            &mut port.direction,
                                            PortDirection::Bidirectional,
                                            "Bidirectional",
                                        );
                                        ui.selectable_value(
                                            &mut port.direction,
                                            PortDirection::Power,
                                            "Power",
                                        );
                                        ui.selectable_value(
                                            &mut port.direction,
                                            PortDirection::Ground,
                                            "Ground",
                                        );
                                    });

                                // Edge combo
                                egui::ComboBox::from_id_salt(format!("port_edge_{}", idx))
                                    .selected_text(port.edge.display_name())
                                    .width(70.0)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut port.edge,
                                            PortEdge::Left,
                                            "Left",
                                        );
                                        ui.selectable_value(
                                            &mut port.edge,
                                            PortEdge::Right,
                                            "Right",
                                        );
                                        ui.selectable_value(
                                            &mut port.edge,
                                            PortEdge::Top,
                                            "Top",
                                        );
                                        ui.selectable_value(
                                            &mut port.edge,
                                            PortEdge::Bottom,
                                            "Bottom",
                                        );
                                    });

                                ui.label(
                                    RichText::new(format!("-> {}", port.internal_net))
                                        .font(egui::FontId::monospace(10.0))
                                        .color(Color32::from_rgb(140, 160, 180)),
                                );

                                if ui.small_button("X").clicked() {
                                    to_remove = Some(idx);
                                }
                            });
                        }

                        if let Some(idx) = to_remove {
                            self.ports.remove(idx);
                        }
                    });

                ui.add_space(6.0);
                ui.checkbox(&mut self.show_spice_preview, "Show SPICE .SUBCKT Preview");

                if self.show_spice_preview {
                    let temp_pkg = SubcircuitPackage::new(
                        &self.package_name,
                        &self.description,
                        self.ports.clone(),
                        self.internal_components.clone(),
                        self.internal_wires.clone(),
                    );
                    let spice_text = temp_pkg.to_spice_subckt();
                    ui.group(|ui| {
                        egui::ScrollArea::vertical()
                            .max_height(100.0)
                            .show(ui, |ui| {
                                ui.monospace(spice_text);
                            });
                    });
                }

                if !self.status_message.is_empty() {
                    ui.label(RichText::new(&self.status_message).color(Color32::from_rgb(255, 100, 100)));
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Package & Register Block").clicked() {
                        let name_trimmed = self.package_name.trim();
                        if name_trimmed.is_empty() {
                            self.status_message = "Package name cannot be empty".to_string();
                        } else {
                            let mut pkg = SubcircuitPackage::new(
                                name_trimmed,
                                &self.description,
                                self.ports.clone(),
                                self.internal_components.clone(),
                                self.internal_wires.clone(),
                            );
                            pkg.author = self.author.clone();
                            action = Some(SubcircuitDialogAction::SavePackage(pkg));
                            self.is_open = false;
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        action = Some(SubcircuitDialogAction::Cancel);
                        self.is_open = false;
                    }
                });
            });

        if !open {
            self.is_open = false;
            if action.is_none() {
                action = Some(SubcircuitDialogAction::Cancel);
            }
        }

        action
    }
}
