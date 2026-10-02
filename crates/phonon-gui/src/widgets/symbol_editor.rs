#![deny(unsafe_code)]

//! Logisim/KiCad-inspired interactive custom component symbol and shape editor widget.
//!
//! Provides a visual CAD workbench to design custom component outlines, place directional
//! terminal pins, visually position reference designators and value labels with zero-collision
//! guarantees, and attach underlying SPICE subcircuit macro-model netlist bindings.

use crate::schematic::symbol::{
    CustomComponentSymbol, SymbolPin, SymbolPrimitive, TerminalDirection,
};
use egui::{
    Color32, DragValue, FontId, Pos2, Sense, Stroke, Vec2, Window,
};

/// Active drawing tool in the component symbol editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolEditorTool {
    Select,
    Line,
    Rectangle,
    Circle,
    Arc,
    Pin,
    Text,
}

impl SymbolEditorTool {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Line => "Line",
            Self::Rectangle => "Rectangle",
            Self::Circle => "Circle",
            Self::Arc => "Arc",
            Self::Pin => "Pin",
            Self::Text => "Text",
        }
    }
}

/// Interactive modal dialog for authoring custom component symbols and shapes.
#[derive(Debug, Clone)]
pub struct SymbolEditorDialog {
    pub is_open: bool,
    pub symbol: CustomComponentSymbol,
    pub active_tool: SymbolEditorTool,
    pub selected_prim_idx: Option<usize>,
    pub selected_pin_idx: Option<usize>,
    pub grid_size: f32,
    pub zoom: f32,
    pub pan: Vec2,
    pub draft_start: Option<Pos2>,
    pub status_message: String,
    pub new_pin_name: String,
    pub new_pin_dir: TerminalDirection,
    pub new_text_content: String,
}

impl Default for SymbolEditorDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolEditorDialog {
    /// Creates a new SymbolEditorDialog initialized with a blank symbol template.
    pub fn new() -> Self {
        let mut symbol = CustomComponentSymbol::new("custom_mod", "Custom Module", "U", "Integrated Circuits");
        // Add a default rectangular IC box and two default terminal pins
        symbol.add_primitive(SymbolPrimitive::Rectangle {
            min: [-30.0, -20.0],
            max: [30.0, 20.0],
            filled: false,
            stroke_width: 1.5,
        });
        symbol.add_pin(SymbolPin::new(0, "IN", TerminalDirection::Input, [-40.0, 0.0], 1));
        symbol.add_pin(SymbolPin::new(1, "OUT", TerminalDirection::Output, [40.0, 0.0], 2));

        Self {
            is_open: false,
            symbol,
            active_tool: SymbolEditorTool::Select,
            selected_prim_idx: None,
            selected_pin_idx: None,
            grid_size: 10.0,
            zoom: 2.0,
            pan: Vec2::ZERO,
            draft_start: None,
            status_message: "Ready. Select a tool to begin drawing.".to_string(),
            new_pin_name: "P1".to_string(),
            new_pin_dir: TerminalDirection::Passive,
            new_text_content: "Label".to_string(),
        }
    }

    /// Opens the symbol editor modal populated with the given symbol.
    pub fn open_symbol(&mut self, symbol: CustomComponentSymbol) {
        self.symbol = symbol;
        self.is_open = true;
        self.selected_prim_idx = None;
        self.selected_pin_idx = None;
        self.draft_start = None;
        self.status_message = format!("Editing symbol '{}'", self.symbol.id);
    }

    /// Renders the Symbol Editor window in egui.
    pub fn show(&mut self, ctx: &egui::Context, mut on_save: impl FnMut(CustomComponentSymbol)) {
        if !self.is_open {
            return;
        }

        let mut open = self.is_open;

        Window::new("Component Symbol & Shape Editor")
            .open(&mut open)
            .default_size(Vec2::new(760.0, 560.0))
            .min_size(Vec2::new(600.0, 400.0))
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Symbol Editor");
                    ui.separator();
                    ui.label(format!("ID: {}", self.symbol.id));
                    ui.separator();

                    if ui.button("Save & Apply").clicked() {
                        self.symbol.resolve_label_collisions();
                        on_save(self.symbol.clone());
                        self.is_open = false;
                    }
                    if ui.button("Auto-Avoid Collisions").clicked() {
                        self.symbol.resolve_label_collisions();
                        self.status_message = "Labels repositioned clear of component body.".to_string();
                    }
                    if ui.button("Cancel").clicked() {
                        self.is_open = false;
                    }
                });

                ui.separator();

                // Split into Toolbar, Canvas, and Inspector
                ui.horizontal(|ui| {
                    // Left Tool Palette
                    ui.vertical(|ui| {
                        ui.set_width(140.0);
                        ui.label("Vector Tools:");
                        for tool in [
                            SymbolEditorTool::Select,
                            SymbolEditorTool::Line,
                            SymbolEditorTool::Rectangle,
                            SymbolEditorTool::Circle,
                            SymbolEditorTool::Arc,
                            SymbolEditorTool::Pin,
                            SymbolEditorTool::Text,
                        ] {
                            let selected = self.active_tool == tool;
                            if ui.selectable_label(selected, tool.name()).clicked() {
                                self.active_tool = tool;
                                self.draft_start = None;
                            }
                        }

                        ui.separator();
                        ui.label("Grid & Zoom:");
                        ui.horizontal(|ui| {
                            ui.label("Grid:");
                            ui.add(DragValue::new(&mut self.grid_size).range(2.0..=50.0).suffix("px"));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Zoom:");
                            ui.add(DragValue::new(&mut self.zoom).range(0.5..=10.0).speed(0.1));
                        });

                        ui.separator();
                        ui.label("Pin Presets:");
                        ui.text_edit_singleline(&mut self.new_pin_name);
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut self.new_pin_dir, TerminalDirection::Input, "In");
                            ui.selectable_value(&mut self.new_pin_dir, TerminalDirection::Output, "Out");
                            ui.selectable_value(&mut self.new_pin_dir, TerminalDirection::Bidirectional, "IO");
                            ui.selectable_value(&mut self.new_pin_dir, TerminalDirection::Passive, "Pass");
                        });
                    });

                    ui.separator();

                    // Center Canvas
                    let canvas_size = ui.available_size() - Vec2::new(220.0, 30.0);
                    let (canvas_rect, response) = ui.allocate_exact_size(canvas_size, Sense::click_and_drag());
                    let painter = ui.painter_at(canvas_rect);

                    // Background & Grid
                    painter.rect_filled(canvas_rect, 0.0, Color32::from_rgb(11, 15, 25));
                    let center = canvas_rect.center() + self.pan;

                    // Draw grid dots
                    let step = (self.grid_size * self.zoom).max(5.0);
                    let start_x = canvas_rect.min.x + (center.x - canvas_rect.min.x) % step;
                    let start_y = canvas_rect.min.y + (center.y - canvas_rect.min.y) % step;
                    let mut x = start_x;
                    while x < canvas_rect.max.x {
                        let mut y = start_y;
                        while y < canvas_rect.max.y {
                            painter.circle_filled(Pos2::new(x, y), 0.75, Color32::from_rgb(30, 41, 59));
                            y += step;
                        }
                        x += step;
                    }

                    // Origin Axis lines
                    painter.line_segment(
                        [Pos2::new(canvas_rect.min.x, center.y), Pos2::new(canvas_rect.max.x, center.y)],
                        Stroke::new(1.0, Color32::from_rgb(30, 41, 59)),
                    );
                    painter.line_segment(
                        [Pos2::new(center.x, canvas_rect.min.y), Pos2::new(center.x, canvas_rect.max.y)],
                        Stroke::new(1.0, Color32::from_rgb(30, 41, 59)),
                    );

                    // Draw the symbol primitives and pins
                    self.symbol.draw(&painter, center, self.zoom, 0, Color32::from_rgb(248, 250, 252));

                    // Draw Designator and Value labels
                    let des_pos = center + Vec2::new(self.symbol.labels.designator_offset[0], self.symbol.labels.designator_offset[1]) * self.zoom;
                    let val_pos = center + Vec2::new(self.symbol.labels.value_offset[0], self.symbol.labels.value_offset[1]) * self.zoom;

                    let has_collision = self.symbol.labels.check_collision(self.symbol.body_bounding_box());
                    let label_color = if has_collision {
                        Color32::from_rgb(239, 68, 68) // Red warning for collision
                    } else {
                        Color32::from_rgb(56, 189, 248) // Clean cyan
                    };

                    painter.text(des_pos, egui::Align2::LEFT_TOP, format!("{}_REF", self.symbol.prefix), FontId::monospace(11.0 * self.zoom), label_color);
                    painter.text(val_pos, egui::Align2::LEFT_TOP, "VALUE", FontId::monospace(10.0 * self.zoom), label_color.gamma_multiply(0.8));

                    // Canvas interaction: Mouse panning & Drawing
                    if response.dragged_by(egui::PointerButton::Secondary) || response.dragged_by(egui::PointerButton::Middle) {
                        self.pan += response.drag_delta();
                    } else if response.clicked() {
                        if let Some(hover_pos) = response.hover_pos() {
                            let local_x = (hover_pos.x - center.x) / self.zoom;
                            let local_y = (hover_pos.y - center.y) / self.zoom;
                            let snapped_x = (local_x / self.grid_size).round() * self.grid_size;
                            let snapped_y = (local_y / self.grid_size).round() * self.grid_size;

                            match self.active_tool {
                                SymbolEditorTool::Pin => {
                                    let new_id = self.symbol.pins.len();
                                    self.symbol.add_pin(SymbolPin::new(
                                        new_id,
                                        self.new_pin_name.clone(),
                                        self.new_pin_dir,
                                        [snapped_x, snapped_y],
                                        new_id + 1,
                                    ));
                                    self.status_message = format!("Placed pin '{}' at ({}, {})", self.new_pin_name, snapped_x, snapped_y);
                                }
                                SymbolEditorTool::Text => {
                                    self.symbol.add_primitive(SymbolPrimitive::Text {
                                        position: [snapped_x, snapped_y],
                                        content: self.new_text_content.clone(),
                                        font_size: 11.0,
                                        is_centered: true,
                                    });
                                    self.status_message = format!("Placed text at ({}, {})", snapped_x, snapped_y);
                                }
                                _ => {}
                            }
                        }
                    }

                    ui.separator();

                    // Right Inspector Panel
                    ui.vertical(|ui| {
                        ui.set_width(200.0);
                        ui.label("Properties:");
                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.symbol.display_name);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Prefix:");
                            ui.text_edit_singleline(&mut self.symbol.prefix);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Category:");
                            ui.text_edit_singleline(&mut self.symbol.category);
                        });

                        ui.separator();
                        ui.label("Label Offsets:");
                        ui.horizontal(|ui| {
                            ui.label("Des X:");
                            ui.add(DragValue::new(&mut self.symbol.labels.designator_offset[0]).speed(1.0));
                            ui.label("Y:");
                            ui.add(DragValue::new(&mut self.symbol.labels.designator_offset[1]).speed(1.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Val X:");
                            ui.add(DragValue::new(&mut self.symbol.labels.value_offset[0]).speed(1.0));
                            ui.label("Y:");
                            ui.add(DragValue::new(&mut self.symbol.labels.value_offset[1]).speed(1.0));
                        });

                        if has_collision {
                            ui.colored_label(Color32::from_rgb(239, 68, 68), "Labels collide with body!");
                        } else {
                            ui.colored_label(Color32::from_rgb(34, 197, 94), "Labels collision-free");
                        }

                        ui.separator();
                        ui.label(format!("Pins ({}):", self.symbol.pins.len()));
                        for (idx, pin) in self.symbol.pins.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}: {}", idx, pin.name));
                                ui.label(format!("({}, {})", pin.rel_pos[0], pin.rel_pos[1]));
                            });
                        }

                        ui.separator();
                        ui.label("SPICE Subcircuit Template:");
                        let mut spice_str = self.symbol.subcircuit_spice_template.clone().unwrap_or_default();
                        if ui.text_edit_multiline(&mut spice_str).changed() {
                            self.symbol.subcircuit_spice_template = Some(spice_str);
                        }
                    });
                });

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Status:");
                    ui.label(&self.status_message);
                });
            });

        self.is_open = open;
    }
}
