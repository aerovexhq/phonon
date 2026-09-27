//! Orthogonal Manhattan wire routing, segment management, and junction detection.

use super::canvas::SchematicCanvas;
use egui::{Color32, Painter, Pos2, Stroke};

/// A single linear wire segment between two points in world coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WireSegment {
    pub start: Pos2,
    pub end: Pos2,
}

impl WireSegment {
    pub fn new(start: Pos2, end: Pos2) -> Self {
        Self { start, end }
    }

    /// Length of the segment.
    pub fn length(&self) -> f32 {
        (self.end - self.start).length()
    }

    /// Checks if a point lies on this segment within a tolerance distance.
    pub fn contains_point(&self, p: Pos2, tol: f32) -> bool {
        let min_x = self.start.x.min(self.end.x) - tol;
        let max_x = self.start.x.max(self.end.x) + tol;
        let min_y = self.start.y.min(self.end.y) - tol;
        let max_y = self.start.y.max(self.end.y) + tol;

        if p.x < min_x || p.x > max_x || p.y < min_y || p.y > max_y {
            return false;
        }

        let l2 = (self.end.x - self.start.x).powi(2) + (self.end.y - self.start.y).powi(2);
        if l2 == 0.0 {
            return (p - self.start).length() <= tol;
        }

        let t = (((p.x - self.start.x) * (self.end.x - self.start.x)
            + (p.y - self.start.y) * (self.end.y - self.start.y))
            / l2)
            .clamp(0.0, 1.0);

        let projection = Pos2::new(
            self.start.x + t * (self.end.x - self.start.x),
            self.start.y + t * (self.end.y - self.start.y),
        );

        (p - projection).length() <= tol
    }
}

/// A multi-segment electrical connection wire.
#[derive(Debug, Clone, PartialEq)]
pub struct SchematicWire {
    pub id: usize,
    pub segments: Vec<WireSegment>,
}

impl SchematicWire {
    pub fn new(id: usize, segments: Vec<WireSegment>) -> Self {
        Self { id, segments }
    }

    /// Creates an orthogonal Manhattan route between two points (horizontal then vertical).
    pub fn manhattan_route(id: usize, from: Pos2, to: Pos2) -> Self {
        let mut segments = Vec::new();
        if (from.x - to.x).abs() > 0.1 && (from.y - to.y).abs() > 0.1 {
            let corner = Pos2::new(to.x, from.y);
            segments.push(WireSegment::new(from, corner));
            segments.push(WireSegment::new(corner, to));
        } else if (from.x - to.x).abs() > 0.1 || (from.y - to.y).abs() > 0.1 {
            segments.push(WireSegment::new(from, to));
        }
        Self { id, segments }
    }

    /// Hit testing for wire selection.
    pub fn contains(&self, world_pos: Pos2, tol: f32) -> bool {
        self.segments
            .iter()
            .any(|seg| seg.contains_point(world_pos, tol))
    }

    /// Renders the wire onto the painter.
    pub fn render(&self, painter: &Painter, canvas: &SchematicCanvas, is_selected: bool) {
        let color = if is_selected {
            Color32::from_rgb(255, 180, 50)
        } else {
            Color32::from_rgb(100, 220, 120) // Classic schematic wire green
        };
        let stroke = Stroke::new(2.0 * canvas.zoom.clamp(0.8, 2.0), color);

        for seg in &self.segments {
            let s_screen = canvas.world_to_screen(seg.start);
            let e_screen = canvas.world_to_screen(seg.end);
            painter.line_segment([s_screen, e_screen], stroke);
        }
    }
}

/// Identifies all T-junctions or intersection nodes among wires and pins.
pub fn compute_junction_dots(wires: &[SchematicWire], pins: &[Pos2]) -> Vec<Pos2> {
    let mut junctions = Vec::new();
    let tol = 2.0;

    // Collect all unique candidate points (all segment endpoints and pins)
    let mut candidates = Vec::new();
    for wire in wires {
        for seg in &wire.segments {
            candidates.push(seg.start);
            candidates.push(seg.end);
        }
    }
    for &pin in pins {
        candidates.push(pin);
    }

    for &p in &candidates {
        let mut degree = 0;
        for wire in wires {
            for seg in &wire.segments {
                let touches_start = (p - seg.start).length() <= tol;
                let touches_end = (p - seg.end).length() <= tol;
                if touches_start || touches_end {
                    degree += 1;
                } else if seg.contains_point(p, tol) {
                    // Interior pass-through counts as 2 branches (both sides of the line)
                    degree += 2;
                }
            }
        }

        let touches_pin = pins.iter().any(|&pin| (pin - p).length() <= tol);

        // A visual junction dot is placed if:
        // - 3 or more branches meet (T-junction, cross-junction)
        // - 2 or more branches meet a component pin
        if (degree >= 3 || (degree >= 2 && touches_pin))
            && !junctions.iter().any(|&j: &Pos2| (j - p).length() < tol)
        {
            junctions.push(p);
        }
    }

    junctions
}
