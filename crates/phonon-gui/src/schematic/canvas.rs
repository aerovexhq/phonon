#![deny(unsafe_code)]

//! Infinite CAD schematic canvas with pan, zoom, grid rendering, and coordinate projection.

use super::components::SchematicComponent;
use super::erc::{ErcDiagnostic, ErcSeverity};
use super::wire::SchematicWire;
use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2};

/// Manages infinite vector canvas pan, zoom, coordinate transformations, and visual schematic objects.
#[derive(Debug, Clone)]
pub struct SchematicCanvas {
    pub pan: Vec2,
    pub zoom: f32,
    pub grid_size: f32,
    pub show_grid: bool,
    pub components: Vec<SchematicComponent>,
    pub wires: Vec<SchematicWire>,
}

impl Default for SchematicCanvas {
    fn default() -> Self {
        Self {
            pan: Vec2::new(100.0, 100.0),
            zoom: 1.0,
            grid_size: 20.0,
            show_grid: true,
            components: Vec::new(),
            wires: Vec::new(),
        }
    }
}

impl SchematicCanvas {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a schematic component to the canvas.
    pub fn add_component(&mut self, comp: SchematicComponent) {
        self.components.push(comp);
    }

    /// Adds a wire routing segment to the canvas.
    pub fn add_wire(&mut self, wire: SchematicWire) {
        self.wires.push(wire);
    }

    /// Clears all schematic components and wires from the canvas.
    pub fn clear(&mut self) {
        self.components.clear();
        self.wires.clear();
    }

    /// Renders visual ERC diagnostic overlay with color-coded pulsing rings and hover tooltips.
    pub fn draw(
        &self,
        painter: &Painter,
        erc_diagnostics: &[ErcDiagnostic],
        show_erc_overlay: bool,
        hover_pos: Option<Pos2>,
    ) {
        if !show_erc_overlay || erc_diagnostics.is_empty() {
            return;
        }

        let time = painter.ctx().input(|i| i.time);
        let pulse = ((time * 4.0).sin() * 0.2 + 0.9) as f32;
        let base_radius = 12.0 * self.zoom.clamp(0.8, 2.0);
        let ring_radius = base_radius * pulse;

        for diagnostic in erc_diagnostics {
            let (ring_color, badge_color) = match diagnostic.severity {
                ErcSeverity::Error => (
                    Color32::from_rgb(220, 50, 50),
                    Color32::from_rgba_unmultiplied(220, 50, 50, 180),
                ),
                ErcSeverity::Warning => (
                    Color32::from_rgb(230, 160, 20),
                    Color32::from_rgba_unmultiplied(230, 160, 20, 180),
                ),
                ErcSeverity::Info => (
                    Color32::from_rgb(50, 150, 240),
                    Color32::from_rgba_unmultiplied(50, 150, 240, 180),
                ),
            };

            for &pin_pos in &diagnostic.pin_positions {
                let screen_pos = self.world_to_screen(pin_pos);

                // Pulsing diagnostic ring
                painter.circle_stroke(
                    screen_pos,
                    ring_radius,
                    Stroke::new(2.5, ring_color),
                );

                // Center indicator badge
                painter.circle_filled(
                    screen_pos,
                    4.0 * self.zoom.clamp(0.8, 1.5),
                    badge_color,
                );

                // Hover tooltip check
                if let Some(mouse) = hover_pos {
                    if (mouse - screen_pos).length() <= (ring_radius + 6.0) {
                        let text = diagnostic.message.clone();
                        let font_id = FontId::proportional(11.0);
                        let galley = painter.layout_no_wrap(
                            text,
                            font_id,
                            Color32::from_rgb(240, 245, 250),
                        );
                        let tooltip_rect = Rect::from_min_size(
                            screen_pos + Vec2::new(12.0, -18.0),
                            galley.size() + Vec2::new(12.0, 8.0),
                        );
                        painter.rect_filled(
                            tooltip_rect,
                            3.0,
                            Color32::from_rgba_unmultiplied(25, 30, 42, 240),
                        );
                        painter.rect_stroke(
                            tooltip_rect,
                            3.0,
                            Stroke::new(1.0, ring_color),
                            StrokeKind::Outside,
                        );
                        painter.galley(
                            tooltip_rect.min + Vec2::new(6.0, 4.0),
                            galley,
                            Color32::from_rgb(240, 245, 250),
                        );
                    }
                }
            }
        }

        // Request continuous repaint for smooth pulsing animation
        painter.ctx().request_repaint();
    }

    /// Indicates whether canvas text selection lockout is enforced.
    #[inline]
    pub fn is_text_selection_locked(&self) -> bool {
        true
    }

    /// Transforms world/schematic coordinates to screen pixel coordinates.
    #[inline]
    pub fn world_to_screen(&self, world_pos: Pos2) -> Pos2 {
        Pos2::new(
            self.pan.x + world_pos.x * self.zoom,
            self.pan.y + world_pos.y * self.zoom,
        )
    }

    /// Transforms screen pixel coordinates to world/schematic coordinates.
    #[inline]
    pub fn screen_to_world(&self, screen_pos: Pos2) -> Pos2 {
        Pos2::new(
            (screen_pos.x - self.pan.x) / self.zoom,
            (screen_pos.y - self.pan.y) / self.zoom,
        )
    }

    /// Snaps world coordinates to the nearest grid intersection point.
    #[inline]
    pub fn snap_to_grid(&self, world_pos: Pos2) -> Pos2 {
        let x = (world_pos.x / self.grid_size).round() * self.grid_size;
        let y = (world_pos.y / self.grid_size).round() * self.grid_size;
        Pos2::new(x, y)
    }

    /// Smoothly updates canvas zoom with continuous exponential scaling centered at an optional screen position.
    pub fn apply_zoom_delta(&mut self, scroll_delta: f32, focus_pos: Option<Pos2>) {
        if scroll_delta.abs() > 0.0 {
            let clamped_delta = scroll_delta.clamp(-120.0, 120.0);
            let zoom_factor = (clamped_delta * 0.0015).exp();
            let new_zoom = (self.zoom * zoom_factor).clamp(0.2, 5.0);
            if let Some(mouse_pos) = focus_pos {
                let world_before = self.screen_to_world(mouse_pos);
                self.zoom = new_zoom;
                let world_after = self.screen_to_world(mouse_pos);
                self.pan += (world_after - world_before) * self.zoom;
            } else {
                self.zoom = new_zoom;
            }
        }
    }

    /// Handles pan (drag) and zoom (scroll) interactions within the canvas rect.
    pub fn handle_pan_zoom(&mut self, ui: &egui::Ui, response: &egui::Response) {
        // Pan via middle mouse drag or secondary (right) drag
        if response.dragged_by(egui::PointerButton::Middle)
            || response.dragged_by(egui::PointerButton::Secondary)
        {
            self.pan += response.drag_delta();
        }

        // Zoom via mouse scroll wheel centered at cursor
        if response.hovered() {
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll_delta.abs() > 0.0 {
                let mouse_pos = ui.input(|i| i.pointer.hover_pos());
                self.apply_zoom_delta(scroll_delta, mouse_pos);
            }
        }
    }

    /// Draws the background grid (minor dots and major lines) onto the painter.
    pub fn render_grid(&self, painter: &Painter, viewport: Rect) {
        if !self.show_grid {
            return;
        }

        let top_left_world = self.screen_to_world(viewport.min);
        let bottom_right_world = self.screen_to_world(viewport.max);

        let start_x = (top_left_world.x / self.grid_size).floor() as i32;
        let end_x = (bottom_right_world.x / self.grid_size).ceil() as i32;
        let start_y = (top_left_world.y / self.grid_size).floor() as i32;
        let end_y = (bottom_right_world.y / self.grid_size).ceil() as i32;

        let dot_color = Color32::from_rgba_unmultiplied(120, 140, 160, 45);
        let major_line_color = Color32::from_rgba_unmultiplied(100, 130, 160, 25);
        let major_stroke = Stroke::new(1.0, major_line_color);

        // Major grid lines every 5 units
        for gx in start_x..=end_x {
            if gx % 5 == 0 {
                let x_screen = self
                    .world_to_screen(Pos2::new(gx as f32 * self.grid_size, 0.0))
                    .x;
                if viewport.x_range().contains(x_screen) {
                    painter.line_segment(
                        [
                            Pos2::new(x_screen, viewport.min.y),
                            Pos2::new(x_screen, viewport.max.y),
                        ],
                        major_stroke,
                    );
                }
            }
        }

        for gy in start_y..=end_y {
            if gy % 5 == 0 {
                let y_screen = self
                    .world_to_screen(Pos2::new(0.0, gy as f32 * self.grid_size))
                    .y;
                if viewport.y_range().contains(y_screen) {
                    painter.line_segment(
                        [
                            Pos2::new(viewport.min.x, y_screen),
                            Pos2::new(viewport.max.x, y_screen),
                        ],
                        major_stroke,
                    );
                }
            }
        }

        // Minor grid dots
        if self.zoom >= 0.5 {
            for gx in start_x..=end_x {
                for gy in start_y..=end_y {
                    let world_pos =
                        Pos2::new(gx as f32 * self.grid_size, gy as f32 * self.grid_size);
                    let screen_pos = self.world_to_screen(world_pos);
                    if viewport.contains(screen_pos) {
                        painter.circle_filled(
                            screen_pos,
                            1.2 * self.zoom.clamp(0.8, 1.5),
                            dot_color,
                        );
                    }
                }
            }
        }
    }
}
