//! Visual schematic components, pin geometries, symbol rendering, and hit testing.

use super::canvas::SchematicCanvas;
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};

/// The electrical device type of a visual schematic component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentKind {
    Resistor,
    Capacitor,
    Inductor,
    VoltageSource,
    CurrentSource,
    Ground,
    Diode,
    Nmos,
    Pmos,
    BjtNpn,
    BjtPnp,
}

impl ComponentKind {
    pub fn prefix(&self) -> &'static str {
        match self {
            Self::Resistor => "R",
            Self::Capacitor => "C",
            Self::Inductor => "L",
            Self::VoltageSource => "V",
            Self::CurrentSource => "I",
            Self::Ground => "GND",
            Self::Diode => "D",
            Self::Nmos => "M",
            Self::Pmos => "M",
            Self::BjtNpn => "Q",
            Self::BjtPnp => "Q",
        }
    }

    pub fn default_value(&self) -> &'static str {
        match self {
            Self::Resistor => "1k",
            Self::Capacitor => "100n",
            Self::Inductor => "10u",
            Self::VoltageSource => "5.0",
            Self::CurrentSource => "1m",
            Self::Ground => "0",
            Self::Diode => "1N4148",
            Self::Nmos => "NMOS_MOD",
            Self::Pmos => "PMOS_MOD",
            Self::BjtNpn => "2N2222",
            Self::BjtPnp => "2N3906",
        }
    }

    /// Returns the local pin offsets relative to component origin (at rotation 0).
    pub fn pin_definitions(&self) -> Vec<(&'static str, Vec2)> {
        match self {
            Self::Resistor | Self::Capacitor | Self::Inductor => {
                vec![("1", Vec2::new(0.0, -40.0)), ("2", Vec2::new(0.0, 40.0))]
            }
            Self::VoltageSource | Self::CurrentSource => {
                vec![("+", Vec2::new(0.0, -40.0)), ("-", Vec2::new(0.0, 40.0))]
            }
            Self::Ground => vec![("GND", Vec2::new(0.0, -20.0))],
            Self::Diode => vec![("A", Vec2::new(0.0, -40.0)), ("K", Vec2::new(0.0, 40.0))],
            Self::Nmos | Self::Pmos => vec![
                ("D", Vec2::new(20.0, -40.0)),
                ("G", Vec2::new(-20.0, 0.0)),
                ("S", Vec2::new(20.0, 40.0)),
            ],
            Self::BjtNpn | Self::BjtPnp => vec![
                ("C", Vec2::new(20.0, -40.0)),
                ("B", Vec2::new(-20.0, 0.0)),
                ("E", Vec2::new(20.0, 40.0)),
            ],
        }
    }
}

/// A visual component placed on the schematic canvas.
#[derive(Debug, Clone, PartialEq)]
pub struct SchematicComponent {
    pub id: usize,
    pub name: String,
    pub kind: ComponentKind,
    pub pos: Pos2,
    /// Rotation in increments of 90 degrees (0 = 0°, 1 = 90°, 2 = 180°, 3 = 270°).
    pub rotation: u8,
    pub value_str: String,
    pub model_name: Option<String>,
}

impl SchematicComponent {
    pub fn new(id: usize, kind: ComponentKind, pos: Pos2, count: usize) -> Self {
        let name = format!("{}{}", kind.prefix(), count);
        let value_str = kind.default_value().to_string();
        Self {
            id,
            name,
            kind,
            pos,
            rotation: 0,
            value_str,
            model_name: None,
        }
    }

    /// Rotates the component clockwise by 90 degrees.
    pub fn rotate_clockwise(&mut self) {
        self.rotation = (self.rotation + 1) % 4;
    }

    /// Transforms a local vector according to the component's rotation.
    fn rotate_vec(&self, v: Vec2) -> Vec2 {
        match self.rotation % 4 {
            0 => v,
            1 => Vec2::new(-v.y, v.x),  // 90 deg CW
            2 => Vec2::new(-v.x, -v.y), // 180 deg
            3 => Vec2::new(v.y, -v.x),  // 270 deg
            _ => v,
        }
    }

    /// Returns the world position of pin at `pin_idx`.
    pub fn pin_world_pos(&self, pin_idx: usize) -> Option<Pos2> {
        let pins = self.kind.pin_definitions();
        let &(_, local_offset) = pins.get(pin_idx)?;
        let rotated = self.rotate_vec(local_offset);
        Some(self.pos + rotated)
    }

    /// Returns all pin names and their world coordinates.
    pub fn all_pins(&self) -> Vec<(&'static str, Pos2)> {
        let pins = self.kind.pin_definitions();
        pins.iter()
            .map(|&(name, offset)| (name, self.pos + self.rotate_vec(offset)))
            .collect()
    }

    /// Hit-test: checks if a world position lies within the component's bounding box.
    pub fn contains(&self, world_pos: Pos2) -> bool {
        let bbox = Rect::from_center_size(self.pos, Vec2::new(60.0, 60.0));
        bbox.contains(world_pos)
    }

    /// Renders the component schematic symbol onto the screen painter.
    pub fn render(
        &self,
        painter: &Painter,
        canvas: &SchematicCanvas,
        is_selected: bool,
        node_voltages: Option<&[(&str, f64)]>,
    ) {
        let stroke_color = if is_selected {
            Color32::from_rgb(255, 180, 50)
        } else {
            Color32::from_rgb(220, 230, 240)
        };
        let stroke = Stroke::new(2.0 * canvas.zoom.clamp(0.8, 2.0), stroke_color);

        // Helper to transform local component coords to screen
        let to_screen = |lx: f32, ly: f32| -> Pos2 {
            let rotated = self.rotate_vec(Vec2::new(lx, ly));
            canvas.world_to_screen(self.pos + rotated)
        };

        match self.kind {
            ComponentKind::Resistor => {
                // IEEE Zig-Zag Resistor
                let pts = [
                    to_screen(0.0, -40.0),
                    to_screen(0.0, -20.0),
                    to_screen(-8.0, -15.0),
                    to_screen(8.0, -5.0),
                    to_screen(-8.0, 5.0),
                    to_screen(8.0, 15.0),
                    to_screen(0.0, 20.0),
                    to_screen(0.0, 40.0),
                ];
                for i in 0..pts.len() - 1 {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }
            }
            ComponentKind::Capacitor => {
                // Two parallel plates
                painter.line_segment([to_screen(0.0, -40.0), to_screen(0.0, -6.0)], stroke);
                painter.line_segment([to_screen(-14.0, -6.0), to_screen(14.0, -6.0)], stroke);
                painter.line_segment([to_screen(-14.0, 6.0), to_screen(14.0, 6.0)], stroke);
                painter.line_segment([to_screen(0.0, 6.0), to_screen(0.0, 40.0)], stroke);
            }
            ComponentKind::Inductor => {
                // 3 coiled semicircles
                painter.line_segment([to_screen(0.0, -40.0), to_screen(0.0, -20.0)], stroke);
                let arcs = [(-10.0, 10.0), (0.0, 10.0), (10.0, 10.0)];
                for &(cy, r) in &arcs {
                    let center = to_screen(0.0, cy);
                    painter.circle_stroke(center, r * canvas.zoom, stroke);
                }
                painter.line_segment([to_screen(0.0, 20.0), to_screen(0.0, 40.0)], stroke);
            }
            ComponentKind::VoltageSource => {
                // Circle with + and - signs
                painter.line_segment([to_screen(0.0, -40.0), to_screen(0.0, -20.0)], stroke);
                let center = to_screen(0.0, 0.0);
                painter.circle_stroke(center, 20.0 * canvas.zoom, stroke);
                // Plus sign
                painter.line_segment([to_screen(-4.0, -10.0), to_screen(4.0, -10.0)], stroke);
                painter.line_segment([to_screen(0.0, -14.0), to_screen(0.0, -6.0)], stroke);
                // Minus sign
                painter.line_segment([to_screen(-4.0, 10.0), to_screen(4.0, 10.0)], stroke);
                painter.line_segment([to_screen(0.0, 20.0), to_screen(0.0, 40.0)], stroke);
            }
            ComponentKind::CurrentSource => {
                // Circle with directional arrow
                painter.line_segment([to_screen(0.0, -40.0), to_screen(0.0, -20.0)], stroke);
                let center = to_screen(0.0, 0.0);
                painter.circle_stroke(center, 20.0 * canvas.zoom, stroke);
                // Arrow
                painter.line_segment([to_screen(0.0, 10.0), to_screen(0.0, -10.0)], stroke);
                painter.line_segment([to_screen(-4.0, -4.0), to_screen(0.0, -10.0)], stroke);
                painter.line_segment([to_screen(4.0, -4.0), to_screen(0.0, -10.0)], stroke);
                painter.line_segment([to_screen(0.0, 20.0), to_screen(0.0, 40.0)], stroke);
            }
            ComponentKind::Ground => {
                // Ground triangle / 3 horizontal bars
                painter.line_segment([to_screen(0.0, -20.0), to_screen(0.0, 0.0)], stroke);
                painter.line_segment([to_screen(-15.0, 0.0), to_screen(15.0, 0.0)], stroke);
                painter.line_segment([to_screen(-10.0, 6.0), to_screen(10.0, 6.0)], stroke);
                painter.line_segment([to_screen(-5.0, 12.0), to_screen(5.0, 12.0)], stroke);
            }
            ComponentKind::Diode => {
                // Triangle pointing to cathode bar
                painter.line_segment([to_screen(0.0, -40.0), to_screen(0.0, -10.0)], stroke);
                let tri = [
                    to_screen(-12.0, -10.0),
                    to_screen(12.0, -10.0),
                    to_screen(0.0, 10.0),
                ];
                painter.line_segment([tri[0], tri[1]], stroke);
                painter.line_segment([tri[1], tri[2]], stroke);
                painter.line_segment([tri[2], tri[0]], stroke);
                // Cathode bar
                painter.line_segment([to_screen(-12.0, 10.0), to_screen(12.0, 10.0)], stroke);
                painter.line_segment([to_screen(0.0, 10.0), to_screen(0.0, 40.0)], stroke);
            }
            ComponentKind::Nmos | ComponentKind::Pmos => {
                // Gate bar
                painter.line_segment([to_screen(-20.0, 0.0), to_screen(-8.0, 0.0)], stroke);
                painter.line_segment([to_screen(-8.0, -18.0), to_screen(-8.0, 18.0)], stroke);
                // Channel bars
                painter.line_segment([to_screen(0.0, -15.0), to_screen(0.0, 15.0)], stroke);
                // Drain & Source
                painter.line_segment([to_screen(0.0, -12.0), to_screen(20.0, -12.0)], stroke);
                painter.line_segment([to_screen(20.0, -12.0), to_screen(20.0, -40.0)], stroke);
                painter.line_segment([to_screen(0.0, 12.0), to_screen(20.0, 12.0)], stroke);
                painter.line_segment([to_screen(20.0, 12.0), to_screen(20.0, 40.0)], stroke);
            }
            ComponentKind::BjtNpn | ComponentKind::BjtPnp => {
                // Base
                painter.line_segment([to_screen(-20.0, 0.0), to_screen(0.0, 0.0)], stroke);
                painter.line_segment([to_screen(0.0, -15.0), to_screen(0.0, 15.0)], stroke);
                // Collector & Emitter branches
                painter.line_segment([to_screen(0.0, -8.0), to_screen(20.0, -40.0)], stroke);
                painter.line_segment([to_screen(0.0, 8.0), to_screen(20.0, 40.0)], stroke);
            }
        }

        // Draw pin snap dots
        let pin_color = Color32::from_rgb(80, 200, 255);
        for (_, p_world) in self.all_pins() {
            let p_screen = canvas.world_to_screen(p_world);
            painter.circle_filled(p_screen, 3.5 * canvas.zoom.clamp(0.8, 1.4), pin_color);
        }

        // Draw labels (Name and Value)
        let label_pos = canvas.world_to_screen(self.pos + Vec2::new(18.0, -10.0));
        let val_pos = canvas.world_to_screen(self.pos + Vec2::new(18.0, 6.0));
        let font_size = 12.0 * canvas.zoom.clamp(0.8, 1.8);

        painter.text(
            label_pos,
            Align2::LEFT_CENTER,
            &self.name,
            FontId::proportional(font_size),
            if is_selected {
                Color32::from_rgb(255, 200, 80)
            } else {
                Color32::from_rgb(180, 220, 255)
            },
        );

        if self.kind != ComponentKind::Ground {
            painter.text(
                val_pos,
                Align2::LEFT_CENTER,
                &self.value_str,
                FontId::proportional(font_size * 0.9),
                Color32::from_rgb(170, 185, 200),
            );
        }

        // Optional live voltage readout badges near pins
        if let Some(voltages) = node_voltages {
            for (pin_name, pin_pos) in self.all_pins() {
                if let Some(&(_, v)) = voltages.iter().find(|(name, _)| *name == pin_name) {
                    let badge_pos = canvas.world_to_screen(pin_pos + Vec2::new(8.0, -8.0));
                    painter.text(
                        badge_pos,
                        Align2::LEFT_CENTER,
                        format!("{:.2}V", v),
                        FontId::monospace(10.0 * canvas.zoom.clamp(0.8, 1.5)),
                        Color32::from_rgb(100, 255, 160),
                    );
                }
            }
        }
    }
}
