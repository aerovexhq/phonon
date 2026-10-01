#![deny(unsafe_code)]

//! High-density vectorized bus routing engine, bit tap-offs, and visual width decorators.

use super::canvas::SchematicCanvas;
use super::wire::WireSegment;
use egui::{Align2, Color32, FontId, Painter, Pos2, Stroke, Vec2};

/// Multi-bit digital or analog vectorized bus signal specification.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BusSignal {
    pub base_name: String,
    pub width: u16,
    pub msb: u16,
    pub lsb: u16,
}

impl BusSignal {
    /// Creates a new vectorized bus signal with MSB and LSB bit indices.
    pub fn new(base_name: impl Into<String>, msb: u16, lsb: u16) -> Self {
        let width = if msb >= lsb {
            msb - lsb + 1
        } else {
            lsb - msb + 1
        };
        Self {
            base_name: base_name.into(),
            width,
            msb,
            lsb,
        }
    }

    /// Formats the standard bus label representation (e.g. "DATA[31:0]").
    pub fn format_label(&self) -> String {
        format!("{}[{}:{}]", self.base_name, self.msb, self.lsb)
    }

    /// Resolves the individual net name at the specified bit index if within bus range.
    pub fn net_at_index(&self, index: u16) -> Option<String> {
        let (min, max) = if self.msb >= self.lsb {
            (self.lsb, self.msb)
        } else {
            (self.msb, self.lsb)
        };
        if index >= min && index <= max {
            Some(format!("{}[{}]", self.base_name, index))
        } else {
            None
        }
    }
}

/// A breakout tap-off extracting an individual bit from a high-density bus.
#[derive(Debug, Clone, PartialEq)]
pub struct BusTapOff {
    pub bit_index: u16,
    pub tap_pos: Pos2,
    pub breakout_wire_id: usize,
    pub breakout_net: String,
}

/// A high-density vectorized bus route containing multiple orthogonal segments and tap-offs.
#[derive(Debug, Clone, PartialEq)]
pub struct SchematicBus {
    pub id: usize,
    pub signal: BusSignal,
    pub segments: Vec<WireSegment>,
    pub stroke_width: f32,
    pub tap_offs: Vec<BusTapOff>,
}

impl SchematicBus {
    /// Constructs a new schematic bus with default 3.5 px stroke width.
    pub fn new(id: usize, signal: BusSignal, segments: Vec<WireSegment>) -> Self {
        Self {
            id,
            signal,
            segments,
            stroke_width: 3.5,
            tap_offs: Vec::new(),
        }
    }

    /// Adds a bit tap-off extraction point to this bus.
    pub fn add_tap_off(
        &mut self,
        bit: u16,
        pos: Pos2,
        wire_id: usize,
    ) -> Result<BusTapOff, String> {
        match self.signal.net_at_index(bit) {
            Some(breakout_net) => {
                let tap = BusTapOff {
                    bit_index: bit,
                    tap_pos: pos,
                    breakout_wire_id: wire_id,
                    breakout_net,
                };
                self.tap_offs.push(tap.clone());
                Ok(tap)
            }
            None => Err(format!(
                "Bit index {} out of range for bus {}",
                bit,
                self.signal.format_label()
            )),
        }
    }

    /// Checks if a world position lies on any segment of this bus within tolerance.
    pub fn contains_point(&self, p: Pos2, tol: f32) -> bool {
        self.segments.iter().any(|s| s.contains_point(p, tol))
    }

    /// Visual bus rendering with wide stroke, diagonal slash width decorator, and numeral badge.
    pub fn render(&self, painter: &Painter, canvas: &SchematicCanvas, is_selected: bool) {
        let stroke_color = if is_selected {
            Color32::from_rgb(100, 200, 255)
        } else {
            Color32::from_rgb(0, 160, 210)
        };
        let effective_width = self.stroke_width * canvas.zoom.clamp(0.6, 2.0);
        let bus_stroke = Stroke::new(effective_width, stroke_color);

        for seg in &self.segments {
            let s = canvas.world_to_screen(seg.start);
            let e = canvas.world_to_screen(seg.end);
            painter.line_segment([s, e], bus_stroke);
        }

        // Render diagonal slash `/` width decorator and numeral badge on longest segment
        if let Some(longest) = self.segments.iter().max_by(|a, b| {
            a.length()
                .partial_cmp(&b.length())
                .unwrap_or(std::cmp::Ordering::Equal)
        }) {
            if longest.length() > 10.0 {
                let mid_world = Pos2::new(
                    (longest.start.x + longest.end.x) * 0.5,
                    (longest.start.y + longest.end.y) * 0.5,
                );
                let mid_screen = canvas.world_to_screen(mid_world);
                let slash_len = 7.0 * canvas.zoom.clamp(0.8, 1.5);
                let slash_p1 = mid_screen + Vec2::new(-slash_len, slash_len);
                let slash_p2 = mid_screen + Vec2::new(slash_len, -slash_len);
                painter.line_segment(
                    [slash_p1, slash_p2],
                    Stroke::new(2.0, Color32::from_rgb(220, 240, 255)),
                );

                let badge_text = format!("/{}", self.signal.width);
                painter.text(
                    mid_screen + Vec2::new(8.0, -10.0),
                    Align2::LEFT_CENTER,
                    badge_text,
                    FontId::monospace(11.0 * canvas.zoom.clamp(0.8, 1.3)),
                    Color32::from_rgb(180, 230, 255),
                );
            }
        }

        // Render tap offs
        for tap in &self.tap_offs {
            let p_screen = canvas.world_to_screen(tap.tap_pos);
            painter.circle_filled(
                p_screen,
                4.0 * canvas.zoom.clamp(0.8, 1.5),
                Color32::from_rgb(0, 220, 255),
            );
            painter.text(
                p_screen + Vec2::new(6.0, -6.0),
                Align2::LEFT_BOTTOM,
                &tap.breakout_net,
                FontId::monospace(10.0 * canvas.zoom.clamp(0.8, 1.3)),
                Color32::from_rgb(150, 215, 255),
            );
        }
    }
}
