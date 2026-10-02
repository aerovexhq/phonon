#![deny(unsafe_code)]

//! Industry-grade interactive component symbol and shape design architecture.
//!
//! Provides vector drawing primitives (lines, rectangles, circles, arcs, polygons, text),
//! interactive terminal pin mapping, collision-free text label positioning, and strict UI-level
//! isolation ensuring zero simulation RAM overhead in the numerical solver kernel.

use std::collections::HashMap;
use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2};

/// 2D Vector drawing primitives for component symbol visual outlines.
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolPrimitive {
    /// Line segment connecting two local coordinate points.
    Line {
        start: [f32; 2],
        end: [f32; 2],
        stroke_width: f32,
    },
    /// Rectangle defined by minimum and maximum diagonal coordinates.
    Rectangle {
        min: [f32; 2],
        max: [f32; 2],
        filled: bool,
        stroke_width: f32,
    },
    /// Circle centered at local origin with radius.
    Circle {
        center: [f32; 2],
        radius: f32,
        filled: bool,
        stroke_width: f32,
    },
    /// Circular arc defined between start and end angles (in radians).
    Arc {
        center: [f32; 2],
        radius: f32,
        start_angle_rad: f32,
        end_angle_rad: f32,
        stroke_width: f32,
    },
    /// Closed or open polygon vertices.
    Polygon {
        points: Vec<[f32; 2]>,
        filled: bool,
        stroke_width: f32,
    },
    /// Text annotation or pin identifier label.
    Text {
        position: [f32; 2],
        content: String,
        font_size: f32,
        is_centered: bool,
    },
}

impl SymbolPrimitive {
    /// Computes the axis-aligned bounding box [min_x, min_y, max_x, max_y] of the primitive.
    pub fn bounding_box(&self) -> [f32; 4] {
        match self {
            Self::Line { start, end, .. } => [
                start[0].min(end[0]),
                start[1].min(end[1]),
                start[0].max(end[0]),
                start[1].max(end[1]),
            ],
            Self::Rectangle { min, max, .. } => [
                min[0].min(max[0]),
                min[1].min(max[1]),
                min[0].max(max[0]),
                min[1].max(max[1]),
            ],
            Self::Circle { center, radius, .. } => [
                center[0] - radius,
                center[1] - radius,
                center[0] + radius,
                center[1] + radius,
            ],
            Self::Arc { center, radius, .. } => [
                center[0] - radius,
                center[1] - radius,
                center[0] + radius,
                center[1] + radius,
            ],
            Self::Polygon { points, .. } => {
                if points.is_empty() {
                    return [0.0, 0.0, 0.0, 0.0];
                }
                let mut min_x = points[0][0];
                let mut min_y = points[0][1];
                let mut max_x = points[0][0];
                let mut max_y = points[0][1];
                for pt in points.iter().skip(1) {
                    min_x = min_x.min(pt[0]);
                    min_y = min_y.min(pt[1]);
                    max_x = max_x.max(pt[0]);
                    max_y = max_y.max(pt[1]);
                }
                [min_x, min_y, max_x, max_y]
            }
            Self::Text { position, font_size, content, is_centered } => {
                let approx_width = content.len() as f32 * font_size * 0.6;
                let height = *font_size;
                if *is_centered {
                    [
                        position[0] - approx_width * 0.5,
                        position[1] - height * 0.5,
                        position[0] + approx_width * 0.5,
                        position[1] + height * 0.5,
                    ]
                } else {
                    [
                        position[0],
                        position[1],
                        position[0] + approx_width,
                        position[1] + height,
                    ]
                }
            }
        }
    }

    /// Rotates the primitive by 90-degree increments (0 = 0 deg, 1 = 90 deg, 2 = 180 deg, 3 = 270 deg).
    pub fn rotated(&self, rotation_index: u8) -> Self {
        let rot = rotation_index % 4;
        if rot == 0 {
            return self.clone();
        }

        let rot_pt = |pt: [f32; 2]| -> [f32; 2] {
            match rot {
                1 => [-pt[1], pt[0]],
                2 => [-pt[0], -pt[1]],
                3 => [pt[1], -pt[0]],
                _ => pt,
            }
        };

        match self {
            Self::Line { start, end, stroke_width } => Self::Line {
                start: rot_pt(*start),
                end: rot_pt(*end),
                stroke_width: *stroke_width,
            },
            Self::Rectangle { min, max, filled, stroke_width } => {
                let p1 = rot_pt(*min);
                let p2 = rot_pt(*max);
                Self::Rectangle {
                    min: [p1[0].min(p2[0]), p1[1].min(p2[1])],
                    max: [p1[0].max(p2[0]), p1[1].max(p2[1])],
                    filled: *filled,
                    stroke_width: *stroke_width,
                }
            }
            Self::Circle { center, radius, filled, stroke_width } => Self::Circle {
                center: rot_pt(*center),
                radius: *radius,
                filled: *filled,
                stroke_width: *stroke_width,
            },
            Self::Arc { center, radius, start_angle_rad, end_angle_rad, stroke_width } => {
                let offset = match rot {
                    1 => std::f32::consts::FRAC_PI_2,
                    2 => std::f32::consts::PI,
                    3 => std::f32::consts::FRAC_PI_2 * 3.0,
                    _ => 0.0,
                };
                Self::Arc {
                    center: rot_pt(*center),
                    radius: *radius,
                    start_angle_rad: start_angle_rad + offset,
                    end_angle_rad: end_angle_rad + offset,
                    stroke_width: *stroke_width,
                }
            }
            Self::Polygon { points, filled, stroke_width } => Self::Polygon {
                points: points.iter().map(|p| rot_pt(*p)).collect(),
                filled: *filled,
                stroke_width: *stroke_width,
            },
            Self::Text { position, content, font_size, is_centered } => Self::Text {
                position: rot_pt(*position),
                content: content.clone(),
                font_size: *font_size,
                is_centered: *is_centered,
            },
        }
    }
}

/// Directional designation for an electrical terminal pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerminalDirection {
    Input,
    Output,
    Bidirectional,
    Passive,
}

pub type SymbolPinDirection = TerminalDirection;

impl TerminalDirection {
    /// Returns the display string for the pin direction.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Input => "Input",
            Self::Output => "Output",
            Self::Bidirectional => "Bidirectional",
            Self::Passive => "Passive",
        }
    }
}

/// Electrical terminal pin definition anchored to a component symbol.
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolPin {
    pub id: usize,
    pub name: String,
    pub direction: TerminalDirection,
    pub rel_pos: [f32; 2],
    pub pin_length: f32,
    pub spice_node_index: usize,
}

impl SymbolPin {
    /// Creates a new symbol pin.
    pub fn new(
        id: usize,
        name: impl Into<String>,
        direction: TerminalDirection,
        rel_pos: [f32; 2],
        spice_node_index: usize,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            direction,
            rel_pos,
            pin_length: 10.0,
            spice_node_index,
        }
    }

    /// Rotates the pin anchor relative to the symbol origin by 90-degree steps.
    pub fn rotated(&self, rotation_index: u8) -> Self {
        let rot = rotation_index % 4;
        let rel_pos = match rot {
            1 => [-self.rel_pos[1], self.rel_pos[0]],
            2 => [-self.rel_pos[0], -self.rel_pos[1]],
            3 => [self.rel_pos[1], -self.rel_pos[0]],
            _ => self.rel_pos,
        };
        Self {
            id: self.id,
            name: self.name.clone(),
            direction: self.direction,
            rel_pos,
            pin_length: self.pin_length,
            spice_node_index: self.spice_node_index,
        }
    }
}

/// Customizable placement parameters for component designator (e.g., R1, V1) and value labels.
#[derive(Debug, Clone, PartialEq)]
pub struct LabelPlacement {
    /// Offset [dx, dy] of the reference designator relative to symbol origin.
    pub designator_offset: [f32; 2],
    /// Offset [dx, dy] of the component value label relative to symbol origin.
    pub value_offset: [f32; 2],
    /// Rotation angle in degrees for the labels.
    pub rotation_deg: f32,
    /// Visibility toggle for reference designator.
    pub designator_visible: bool,
    /// Visibility toggle for component value.
    pub value_visible: bool,
}

impl Default for LabelPlacement {
    fn default() -> Self {
        Self {
            designator_offset: [25.0, -15.0],
            value_offset: [25.0, 5.0],
            rotation_deg: 0.0,
            designator_visible: true,
            value_visible: true,
        }
    }
}

impl LabelPlacement {
    /// Checks if a text label bounding box collides with the symbol body bounding box.
    pub fn check_collision(&self, body_bounds: [f32; 4]) -> bool {
        let text_w = 30.0f32;
        let text_h = 12.0f32;

        let des_box = [
            self.designator_offset[0],
            self.designator_offset[1],
            self.designator_offset[0] + text_w,
            self.designator_offset[1] + text_h,
        ];

        let val_box = [
            self.value_offset[0],
            self.value_offset[1],
            self.value_offset[0] + text_w,
            self.value_offset[1] + text_h,
        ];

        let overlaps = |b1: [f32; 4], b2: [f32; 4]| -> bool {
            !(b1[2] <= b2[0] || b1[0] >= b2[2] || b1[3] <= b2[1] || b1[1] >= b2[3])
        };

        (self.designator_visible && overlaps(des_box, body_bounds))
            || (self.value_visible && overlaps(val_box, body_bounds))
    }

    /// Automatically relocates labels to clear the body bounding box, eliminating visual collisions.
    pub fn auto_avoid_collision(&mut self, body_bounds: [f32; 4]) {
        let margin = 8.0f32;
        let target_x = body_bounds[2] + margin;
        self.designator_offset[0] = target_x;
        self.designator_offset[1] = body_bounds[1] + margin;
        self.value_offset[0] = target_x;
        self.value_offset[1] = body_bounds[1] + margin + 18.0;
    }
}

/// Complete user-definable component symbol outline, terminal definition, and macro-model binding.
///
/// Designed with strict UI-simulation decoupling:
/// Simulation engines interact purely with sparse MNA stamps and pin netlists;
/// vector primitives and annotations reside exclusively within this UI structure.
#[derive(Debug, Clone, PartialEq)]
pub struct CustomComponentSymbol {
    pub id: String,
    pub display_name: String,
    pub prefix: String,
    pub category: String,
    pub description: String,
    pub primitives: Vec<SymbolPrimitive>,
    pub pins: Vec<SymbolPin>,
    pub labels: LabelPlacement,
    pub subcircuit_spice_template: Option<String>,
}

impl CustomComponentSymbol {
    /// Creates a new custom component symbol with default label placement.
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        prefix: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            prefix: prefix.into(),
            category: category.into(),
            description: String::new(),
            primitives: Vec::new(),
            pins: Vec::new(),
            labels: LabelPlacement::default(),
            subcircuit_spice_template: None,
        }
    }

    /// Adds a vector drawing primitive.
    pub fn add_primitive(&mut self, primitive: SymbolPrimitive) {
        self.primitives.push(primitive);
    }

    /// Adds an electrical terminal pin.
    pub fn add_pin(&mut self, pin: SymbolPin) {
        self.pins.push(pin);
    }

    /// Computes the collective bounding box of all geometric primitives.
    pub fn body_bounding_box(&self) -> [f32; 4] {
        if self.primitives.is_empty() {
            return [-20.0, -20.0, 20.0, 20.0];
        }
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for prim in &self.primitives {
            let b = prim.bounding_box();
            min_x = min_x.min(b[0]);
            min_y = min_y.min(b[1]);
            max_x = max_x.max(b[2]);
            max_y = max_y.max(b[3]);
        }

        [min_x, min_y, max_x, max_y]
    }

    /// Automatically repositions labels to avoid colliding with any body geometry.
    pub fn resolve_label_collisions(&mut self) {
        let bounds = self.body_bounding_box();
        if self.labels.check_collision(bounds) {
            self.labels.auto_avoid_collision(bounds);
        }
    }

    /// Returns a rotated representation of the symbol.
    pub fn rotated(&self, rotation_index: u8) -> Self {
        let rot = rotation_index % 4;
        if rot == 0 {
            return self.clone();
        }

        let rotated_prims = self.primitives.iter().map(|p| p.rotated(rot)).collect();
        let rotated_pins = self.pins.iter().map(|p| p.rotated(rot)).collect();

        Self {
            id: self.id.clone(),
            display_name: self.display_name.clone(),
            prefix: self.prefix.clone(),
            category: self.category.clone(),
            description: self.description.clone(),
            primitives: rotated_prims,
            pins: rotated_pins,
            labels: self.labels.clone(),
            subcircuit_spice_template: self.subcircuit_spice_template.clone(),
        }
    }

    /// Emits a SPICE subcircuit instance line with pure mathematical netlist node tokens.
    /// Guarantees zero UI vector or graphical payload touches the simulation solver.
    pub fn to_spice_subcircuit_instance(&self, ref_des: &str, connected_nets: &[&str]) -> String {
        let mut netlist_line = format!("X{}", ref_des.trim_start_matches('X'));
        for net in connected_nets {
            netlist_line.push(' ');
            netlist_line.push_str(net);
        }
        netlist_line.push(' ');
        netlist_line.push_str(&self.id);
        netlist_line
    }

    /// Draws the symbol onto an egui painter at the specified screen position and scale.
    pub fn draw(
        &self,
        painter: &Painter,
        origin: Pos2,
        zoom: f32,
        rotation: u8,
        stroke_color: Color32,
    ) {
        let rotated = self.rotated(rotation);

        for prim in &rotated.primitives {
            match prim {
                SymbolPrimitive::Line { start, end, stroke_width } => {
                    let p1 = origin + Vec2::new(start[0], start[1]) * zoom;
                    let p2 = origin + Vec2::new(end[0], end[1]) * zoom;
                    painter.line_segment([p1, p2], Stroke::new(*stroke_width * zoom, stroke_color));
                }
                SymbolPrimitive::Rectangle { min, max, filled, stroke_width } => {
                    let p1 = origin + Vec2::new(min[0], min[1]) * zoom;
                    let p2 = origin + Vec2::new(max[0], max[1]) * zoom;
                    let rect = Rect::from_two_pos(p1, p2);
                    let fill = if *filled { stroke_color.gamma_multiply(0.2) } else { Color32::TRANSPARENT };
                    painter.rect(rect, 0.0, fill, Stroke::new(*stroke_width * zoom, stroke_color), StrokeKind::Middle);
                }
                SymbolPrimitive::Circle { center, radius, filled, stroke_width } => {
                    let c = origin + Vec2::new(center[0], center[1]) * zoom;
                    let r = *radius * zoom;
                    let fill = if *filled { stroke_color.gamma_multiply(0.2) } else { Color32::TRANSPARENT };
                    painter.circle(c, r, fill, Stroke::new(*stroke_width * zoom, stroke_color));
                }
                SymbolPrimitive::Arc { center, radius, start_angle_rad, end_angle_rad, stroke_width } => {
                    let c = origin + Vec2::new(center[0], center[1]) * zoom;
                    let r = *radius * zoom;
                    let steps = 16;
                    let mut prev_pt = None;
                    for i in 0..=steps {
                        let t = i as f32 / steps as f32;
                        let angle = start_angle_rad + (end_angle_rad - start_angle_rad) * t;
                        let pt = c + Vec2::new(angle.cos() * r, angle.sin() * r);
                        if let Some(prev) = prev_pt {
                            painter.line_segment([prev, pt], Stroke::new(*stroke_width * zoom, stroke_color));
                        }
                        prev_pt = Some(pt);
                    }
                }
                SymbolPrimitive::Polygon { points, filled, stroke_width } => {
                    if points.len() >= 2 {
                        let screen_pts: Vec<Pos2> = points
                            .iter()
                            .map(|p| origin + Vec2::new(p[0], p[1]) * zoom)
                            .collect();
                        if *filled {
                            let fill = stroke_color.gamma_multiply(0.2);
                            let shape = egui::Shape::convex_polygon(
                                screen_pts.clone(),
                                fill,
                                Stroke::new(*stroke_width * zoom, stroke_color),
                            );
                            painter.add(shape);
                        } else {
                            for i in 0..screen_pts.len() {
                                let p1 = screen_pts[i];
                                let p2 = screen_pts[(i + 1) % screen_pts.len()];
                                painter.line_segment([p1, p2], Stroke::new(*stroke_width * zoom, stroke_color));
                            }
                        }
                    }
                }
                SymbolPrimitive::Text { position, content, font_size, is_centered } => {
                    let p = origin + Vec2::new(position[0], position[1]) * zoom;
                    let align = if *is_centered { egui::Align2::CENTER_CENTER } else { egui::Align2::LEFT_TOP };
                    painter.text(
                        p,
                        align,
                        content,
                        FontId::proportional(*font_size * zoom),
                        stroke_color,
                    );
                }
            }
        }

        // Draw terminal pins
        for pin in &rotated.pins {
            let pin_pos = origin + Vec2::new(pin.rel_pos[0], pin.rel_pos[1]) * zoom;
            painter.circle_filled(pin_pos, 2.5 * zoom, Color32::from_rgb(56, 189, 248));
        }
    }
}

/// Registry and cache for custom component symbols.
#[derive(Debug, Clone, Default)]
pub struct SymbolLibrary {
    symbols: HashMap<String, CustomComponentSymbol>,
}

impl SymbolLibrary {
    /// Creates an empty symbol library.
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    /// Registers a custom symbol into the library.
    pub fn register(&mut self, symbol: CustomComponentSymbol) {
        self.symbols.insert(symbol.id.clone(), symbol);
    }

    /// Retrieves a symbol by its unique identifier.
    pub fn get(&self, id: &str) -> Option<&CustomComponentSymbol> {
        self.symbols.get(id)
    }

    /// Returns a list of all registered custom symbols.
    pub fn all(&self) -> Vec<&CustomComponentSymbol> {
        self.symbols.values().collect()
    }

    /// Total count of registered symbols.
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Returns true if the library contains no symbols.
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }
}
