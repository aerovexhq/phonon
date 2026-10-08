#![deny(unsafe_code)]

//! Logical net labels, off-sheet connector tags, and visual net labeling.

use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};
use super::canvas::SchematicCanvas;
use crate::theme::PhononTheme;

/// Orientation of the net label tag relative to its anchor point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetLabelOrientation {
    /// Tag extends East (right) of the anchor.
    East,
    /// Tag extends West (left) of the anchor.
    West,
    /// Tag extends North (above) the anchor.
    North,
    /// Tag extends South (below) the anchor.
    South,
}

impl Default for NetLabelOrientation {
    fn default() -> Self {
        Self::East
    }
}

impl NetLabelOrientation {
    /// Cycles orientation clockwise: East -> South -> West -> North -> East.
    pub fn rotate_clockwise(self) -> Self {
        match self {
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
            Self::North => Self::East,
        }
    }

    /// Converts a quarter-turn rotation count (0..=3) to an orientation.
    pub fn from_quarter_turns(turns: u8) -> Self {
        match turns % 4 {
            0 => Self::East,
            1 => Self::South,
            2 => Self::West,
            3 => Self::North,
            _ => unreachable!(),
        }
    }

    /// Returns the quarter-turn count (0..=3) for this orientation.
    pub fn to_quarter_turns(self) -> u8 {
        match self {
            Self::East => 0,
            Self::South => 1,
            Self::West => 2,
            Self::North => 3,
        }
    }
}

/// A logical net label tag attached to a wire, pin, or net at a specific world coordinate.
#[derive(Debug, Clone, PartialEq)]
pub struct NetLabel {
    /// Unique identifier of the label.
    pub id: usize,
    /// Text of the net (e.g. "VCC", "GND", "CLK", "RESET", "DATA0").
    pub name: String,
    /// Anchor position in world coordinates (where it connects to wire/pin).
    pub pos: Pos2,
    /// Direction the tag badge extends from the anchor.
    pub orientation: NetLabelOrientation,
}

impl NetLabel {
    /// Creates a new net label with default East orientation.
    pub fn new(id: usize, name: impl Into<String>, pos: Pos2) -> Self {
        Self {
            id,
            name: name.into(),
            pos,
            orientation: NetLabelOrientation::East,
        }
    }

    /// Sets explicit orientation.
    pub fn with_orientation(mut self, orientation: NetLabelOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Checks if a world position hits this net label (tag body or anchor).
    pub fn hit_test(&self, world_pos: Pos2, tol: f32) -> bool {
        let tag_rect = self.bounding_box();
        tag_rect.expand(tol).contains(world_pos) || (self.pos - world_pos).length() <= tol
    }

    /// Computes the bounding box of the label tag in world coordinates.
    pub fn bounding_box(&self) -> Rect {
        let char_count = self.name.chars().count().max(1);
        let tag_width = (char_count as f32 * 7.5 + 14.0).max(28.0);
        let tag_height = 14.0;

        match self.orientation {
            NetLabelOrientation::East => {
                Rect::from_min_size(self.pos + Vec2::new(2.0, -tag_height * 0.5), Vec2::new(tag_width, tag_height))
            }
            NetLabelOrientation::West => {
                Rect::from_min_size(self.pos + Vec2::new(-tag_width - 2.0, -tag_height * 0.5), Vec2::new(tag_width, tag_height))
            }
            NetLabelOrientation::North => {
                Rect::from_min_size(self.pos + Vec2::new(-tag_width * 0.5, -tag_height - 2.0), Vec2::new(tag_width, tag_height))
            }
            NetLabelOrientation::South => {
                Rect::from_min_size(self.pos + Vec2::new(-tag_width * 0.5, 2.0), Vec2::new(tag_width, tag_height))
            }
        }
    }

    /// Checks if this net label touches the given point within tolerance.
    pub fn touches_point(&self, pt: Pos2, tol: f32) -> bool {
        (self.pos - pt).length() <= tol
    }

    /// Renders the NetLabel onto the painter using the active theme.
    pub fn render(
        &self,
        painter: &Painter,
        canvas: &SchematicCanvas,
        is_selected: bool,
        theme: &PhononTheme,
    ) {
        let anchor_screen = canvas.world_to_screen(self.pos);
        let zoom = canvas.zoom.clamp(0.6, 2.5);

        // 1. Connection anchor dot
        let anchor_color = if is_selected {
            theme.accent_primary
        } else {
            theme.wire_normal
        };
        painter.circle_filled(anchor_screen, 2.5 * zoom, anchor_color);

        // 2. Tag frame geometry in screen space
        let char_count = self.name.chars().count().max(1);
        let font_size = (11.0 * zoom).clamp(8.0, 18.0);
        let pad_x = 6.0 * zoom;
        let pad_y = 2.0 * zoom;
        let tag_w = (char_count as f32 * 6.5 * zoom + pad_x * 2.0).max(24.0 * zoom);
        let tag_h = (font_size + pad_y * 2.0).max(14.0 * zoom);

        let offset = match self.orientation {
            NetLabelOrientation::East => Vec2::new(4.0 * zoom, -tag_h * 0.5),
            NetLabelOrientation::West => Vec2::new(-tag_w - 4.0 * zoom, -tag_h * 0.5),
            NetLabelOrientation::North => Vec2::new(-tag_w * 0.5, -tag_h - 4.0 * zoom),
            NetLabelOrientation::South => Vec2::new(-tag_w * 0.5, 4.0 * zoom),
        };

        let tag_rect = Rect::from_min_size(anchor_screen + offset, Vec2::new(tag_w, tag_h));

        // Background pill
        let bg_color = if is_selected {
            Color32::from_rgba_unmultiplied(
                theme.accent_primary.r(),
                theme.accent_primary.g(),
                theme.accent_primary.b(),
                70,
            )
        } else {
            Color32::from_rgba_unmultiplied(
                theme.component_body.r(),
                theme.component_body.g(),
                theme.component_body.b(),
                230,
            )
        };

        let stroke_color = if is_selected {
            theme.accent_primary
        } else {
            Color32::from_rgba_unmultiplied(
                theme.wire_normal.r(),
                theme.wire_normal.g(),
                theme.wire_normal.b(),
                200,
            )
        };

        painter.rect(
            tag_rect,
            egui::CornerRadius::same(3),
            bg_color,
            Stroke::new(1.0 * zoom, stroke_color),
            egui::StrokeKind::Outside,
        );

        // Leader tick line from anchor to tag edge
        let tick_end = match self.orientation {
            NetLabelOrientation::East => Pos2::new(tag_rect.min.x, anchor_screen.y),
            NetLabelOrientation::West => Pos2::new(tag_rect.max.x, anchor_screen.y),
            NetLabelOrientation::North => Pos2::new(anchor_screen.x, tag_rect.max.y),
            NetLabelOrientation::South => Pos2::new(anchor_screen.x, tag_rect.min.y),
        };
        painter.line_segment([anchor_screen, tick_end], Stroke::new(1.0 * zoom, stroke_color));

        // Text label
        let text_color = if is_selected {
            theme.accent_primary
        } else {
            theme.text_primary
        };
        let galley = painter.layout_no_wrap(
            self.name.clone(),
            FontId::monospace(font_size),
            text_color,
        );
        let text_pos = tag_rect.center() - galley.size() * 0.5;
        painter.galley(text_pos, galley, text_color);
    }

    /// Renders a semi-transparent ghost preview of the NetLabel at the cursor position.
    pub fn render_ghost(
        &self,
        painter: &Painter,
        canvas: &SchematicCanvas,
        theme: &PhononTheme,
    ) {
        let anchor_screen = canvas.world_to_screen(self.pos);
        let zoom = canvas.zoom.clamp(0.6, 2.5);

        // 1. Connection anchor dot
        let anchor_color = theme.accent_primary.gamma_multiply(0.6);
        painter.circle_filled(anchor_screen, 2.5 * zoom, anchor_color);

        // 2. Tag frame geometry in screen space
        let char_count = self.name.chars().count().max(1);
        let font_size = (11.0 * zoom).clamp(8.0, 18.0);
        let pad_x = 6.0 * zoom;
        let pad_y = 2.0 * zoom;
        let tag_w = (char_count as f32 * 6.5 * zoom + pad_x * 2.0).max(24.0 * zoom);
        let tag_h = (font_size + pad_y * 2.0).max(14.0 * zoom);

        let offset = match self.orientation {
            NetLabelOrientation::East => Vec2::new(4.0 * zoom, -tag_h * 0.5),
            NetLabelOrientation::West => Vec2::new(-tag_w - 4.0 * zoom, -tag_h * 0.5),
            NetLabelOrientation::North => Vec2::new(-tag_w * 0.5, -tag_h - 4.0 * zoom),
            NetLabelOrientation::South => Vec2::new(-tag_w * 0.5, 4.0 * zoom),
        };

        let tag_rect = Rect::from_min_size(anchor_screen + offset, Vec2::new(tag_w, tag_h));

        // Semi-transparent ghost background and stroke
        let bg_color = Color32::from_rgba_unmultiplied(
            theme.accent_primary.r(),
            theme.accent_primary.g(),
            theme.accent_primary.b(),
            45,
        );
        let stroke_color = theme.accent_primary.gamma_multiply(0.65);

        painter.rect(
            tag_rect,
            egui::CornerRadius::same(3),
            bg_color,
            Stroke::new(1.0 * zoom, stroke_color),
            egui::StrokeKind::Outside,
        );

        // Leader tick line
        let tick_end = match self.orientation {
            NetLabelOrientation::East => Pos2::new(tag_rect.min.x, anchor_screen.y),
            NetLabelOrientation::West => Pos2::new(tag_rect.max.x, anchor_screen.y),
            NetLabelOrientation::North => Pos2::new(anchor_screen.x, tag_rect.max.y),
            NetLabelOrientation::South => Pos2::new(anchor_screen.x, tag_rect.min.y),
        };
        painter.line_segment([anchor_screen, tick_end], Stroke::new(1.0 * zoom, stroke_color));

        // Text label
        let text_color = theme.accent_primary.gamma_multiply(0.8);
        let galley = painter.layout_no_wrap(
            self.name.clone(),
            FontId::monospace(font_size),
            text_color,
        );
        let text_pos = tag_rect.center() - galley.size() * 0.5;
        painter.galley(text_pos, galley, text_color);
    }
}
