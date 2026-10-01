#![deny(unsafe_code)]

//! Infinite CAD schematic canvas with pan, zoom, grid rendering, and coordinate projection.

use egui::{Color32, Painter, Pos2, Rect, Stroke, Vec2};

/// Manages infinite vector canvas pan, zoom, and coordinate transformations.
#[derive(Debug, Clone)]
pub struct SchematicCanvas {
    pub pan: Vec2,
    pub zoom: f32,
    pub grid_size: f32,
    pub show_grid: bool,
}

impl Default for SchematicCanvas {
    fn default() -> Self {
        Self {
            pan: Vec2::new(100.0, 100.0),
            zoom: 1.0,
            grid_size: 20.0,
            show_grid: true,
        }
    }
}

impl SchematicCanvas {
    pub fn new() -> Self {
        Self::default()
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
