#![deny(unsafe_code)]

//! Infinite CAD schematic canvas with pan, zoom, grid rendering, and coordinate projection.

use super::bus::SchematicBus;
use super::components::SchematicComponent;
use super::erc::{ErcDiagnostic, ErcSeverity};
use super::net_label::NetLabel;
use super::subcircuit::SubcircuitInstance;
use super::wire::SchematicWire;
use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2};

use std::collections::HashSet;

/// Manages infinite vector canvas pan, zoom, coordinate transformations, and visual schematic objects.
#[derive(Debug, Clone, PartialEq)]
pub struct SchematicCanvas {
    pub pan: Vec2,
    pub zoom: f32,
    pub grid_size: f32,
    pub show_grid: bool,
    pub components: Vec<SchematicComponent>,
    pub wires: Vec<SchematicWire>,
    pub subcircuit_instances: Vec<SubcircuitInstance>,
    pub buses: Vec<SchematicBus>,
    pub net_labels: Vec<NetLabel>,
    /// Set of selected component IDs for multi-selection.
    pub selected_component_ids: HashSet<usize>,
    /// Set of selected wire IDs for multi-selection.
    pub selected_wire_ids: HashSet<usize>,
    /// Set of selected net label IDs for multi-selection.
    pub selected_label_ids: HashSet<usize>,
    /// Backward-compatible primary/last selected component ID.
    pub selected_component_id: Option<usize>,
    /// Backward-compatible primary/last selected wire ID.
    pub selected_wire_id: Option<usize>,
    /// Rubberband marquee drag start position in world coordinates.
    pub marquee_start: Option<Pos2>,
    /// Rubberband marquee drag current position in world coordinates.
    pub marquee_current: Option<Pos2>,
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
            subcircuit_instances: Vec::new(),
            buses: Vec::new(),
            net_labels: Vec::new(),
            selected_component_ids: HashSet::new(),
            selected_wire_ids: HashSet::new(),
            selected_label_ids: HashSet::new(),
            selected_component_id: None,
            selected_wire_id: None,
            marquee_start: None,
            marquee_current: None,
        }
    }
}

impl SchematicCanvas {
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks whether the canvas has no components, wires, subcircuit instances, buses, or net labels.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
            && self.wires.is_empty()
            && self.subcircuit_instances.is_empty()
            && self.buses.is_empty()
            && self.net_labels.is_empty()
    }

    /// Resets the canvas coordinate origin to align with the specified screen position,
    /// ensuring that world coordinate (0.0, 0.0) lands exactly at `target_screen`.
    pub fn reset_origin_at_screen(&mut self, target_screen: Pos2) {
        self.pan = target_screen.to_vec2();
    }

    /// Adds a schematic component to the canvas.
    pub fn add_component(&mut self, comp: SchematicComponent) {
        self.components.push(comp);
    }

    /// Adds a wire routing segment to the canvas.
    pub fn add_wire(&mut self, wire: SchematicWire) {
        self.wires.push(wire);
    }

    /// Adds a placed subcircuit instance to the canvas.
    pub fn add_subcircuit_instance(&mut self, inst: SubcircuitInstance) {
        self.subcircuit_instances.push(inst);
    }

    /// Adds a high-density bus route to the canvas.
    pub fn add_bus(&mut self, bus: SchematicBus) {
        self.buses.push(bus);
    }

    /// Adds a logical net label to the canvas.
    pub fn add_net_label(&mut self, label: NetLabel) {
        self.net_labels.push(label);
    }

    /// Clears all schematic components, wires, subcircuit instances, buses, and net labels from the canvas.
    pub fn clear(&mut self) {
        self.components.clear();
        self.wires.clear();
        self.subcircuit_instances.clear();
        self.buses.clear();
        self.net_labels.clear();
        self.clear_selection();
        self.marquee_start = None;
        self.marquee_current = None;
    }

    /// Checks whether a component with the given ID is selected.
    pub fn is_component_selected(&self, id: usize) -> bool {
        self.selected_component_ids.contains(&id) || self.selected_component_id == Some(id)
    }

    /// Checks whether a wire with the given ID is selected.
    pub fn is_wire_selected(&self, id: usize) -> bool {
        self.selected_wire_ids.contains(&id) || self.selected_wire_id == Some(id)
    }

    /// Checks whether a net label with the given ID is selected.
    pub fn is_label_selected(&self, id: usize) -> bool {
        self.selected_label_ids.contains(&id)
    }

    /// Clears all selection state.
    pub fn clear_selection(&mut self) {
        self.selected_component_ids.clear();
        self.selected_wire_ids.clear();
        self.selected_label_ids.clear();
        self.selected_component_id = None;
        self.selected_wire_id = None;
    }

    /// Selects a single net label or adds to selection if multi is true.
    pub fn select_label(&mut self, id: usize, multi: bool) {
        if !multi {
            self.clear_selection();
        }
        self.selected_label_ids.insert(id);
    }

    /// Toggles net label in/out of the selection set.
    pub fn toggle_label_selection(&mut self, id: usize) {
        if self.selected_label_ids.contains(&id) {
            self.selected_label_ids.remove(&id);
        } else {
            self.selected_label_ids.insert(id);
        }
    }

    /// Selects a single component or adds to selection if multi is true.
    pub fn select_component(&mut self, id: usize, multi: bool) {
        if !multi {
            self.clear_selection();
        }
        self.selected_component_ids.insert(id);
        self.selected_component_id = Some(id);
    }

    /// Selects a single wire or adds to selection if multi is true.
    pub fn select_wire(&mut self, id: usize, multi: bool) {
        if !multi {
            self.clear_selection();
        }
        self.selected_wire_ids.insert(id);
        self.selected_wire_id = Some(id);
    }

    /// Toggles a component's selection state (for Shift+Click).
    pub fn toggle_component_selection(&mut self, id: usize) {
        if self.selected_component_ids.contains(&id) {
            self.selected_component_ids.remove(&id);
            if self.selected_component_id == Some(id) {
                self.selected_component_id = self.selected_component_ids.iter().next().copied();
            }
        } else {
            self.selected_component_ids.insert(id);
            self.selected_component_id = Some(id);
        }
    }

    /// Toggles a wire's selection state (for Shift+Click).
    pub fn toggle_wire_selection(&mut self, id: usize) {
        if self.selected_wire_ids.contains(&id) {
            self.selected_wire_ids.remove(&id);
            if self.selected_wire_id == Some(id) {
                self.selected_wire_id = self.selected_wire_ids.iter().next().copied();
            }
        } else {
            self.selected_wire_ids.insert(id);
            self.selected_wire_id = Some(id);
        }
    }

    /// Selects all components and wires on the canvas.
    pub fn select_all(&mut self) {
        self.selected_component_ids = self.components.iter().map(|c| c.id).collect();
        self.selected_wire_ids = self.wires.iter().map(|w| w.id).collect();
        self.selected_component_id = self.selected_component_ids.iter().next().copied();
        self.selected_wire_id = self.selected_wire_ids.iter().next().copied();
    }

    /// Selects all components and wires intersecting the given rectangle in world coordinates.
    /// If `add` is false, clears prior selection before selecting.
    pub fn select_in_rect(&mut self, rect: Rect, add: bool) {
        if !add {
            self.clear_selection();
        }
        for comp in &self.components {
            if comp.intersects_rect(&rect) {
                self.selected_component_ids.insert(comp.id);
                self.selected_component_id = Some(comp.id);
            }
        }
        for wire in &self.wires {
            if wire.intersects_rect(&rect) {
                self.selected_wire_ids.insert(wire.id);
                self.selected_wire_id = Some(wire.id);
            }
        }
    }

    /// Translates all currently selected components and wires by delta in world coordinates.
    /// Also updates any unselected wires connected to the moved components.
    pub fn translate_selection(&mut self, delta: Vec2) {
        if delta == Vec2::ZERO {
            return;
        }

        // Collect old pin positions of moving components to identify attached wires
        let mut moving_pins = Vec::new();
        for comp in &self.components {
            if self.is_component_selected(comp.id) {
                for (_, p) in comp.all_pins() {
                    moving_pins.push(p);
                }
            }
        }

        // 1. Translate selected components
        let sel_comp_ids = self.selected_component_ids.clone();
        let sel_comp_id = self.selected_component_id;
        let is_comp_sel = |id: usize| sel_comp_ids.contains(&id) || sel_comp_id == Some(id);

        for comp in &mut self.components {
            if is_comp_sel(comp.id) {
                comp.pos += delta;
            }
        }

        // 2. Translate selected wires or attached wire endpoints
        let sel_wire_ids = self.selected_wire_ids.clone();
        let sel_wire_id = self.selected_wire_id;
        let is_wire_sel = |id: usize| sel_wire_ids.contains(&id) || sel_wire_id == Some(id);

        for wire in &mut self.wires {
            if is_wire_sel(wire.id) {
                // Entire wire moves with delta
                for seg in &mut wire.segments {
                    seg.start += delta;
                    seg.end += delta;
                }
            } else if !moving_pins.is_empty() {
                // Check if wire endpoints are attached to moving pins
                let start_attached = wire.segments.first().map_or(false, |s| {
                    moving_pins.iter().any(|&p| (p - s.start).length() <= 4.0)
                });
                let end_attached = wire.segments.last().map_or(false, |s| {
                    moving_pins.iter().any(|&p| (p - s.end).length() <= 4.0)
                });

                if start_attached && end_attached {
                    // Both endpoints attached: move whole wire
                    for seg in &mut wire.segments {
                        seg.start += delta;
                        seg.end += delta;
                    }
                } else if start_attached {
                    let old_end = wire.end_point();
                    let new_start = wire.start_point() + delta;
                    let was_vh = wire.segments.first().map_or(false, |s| {
                        (s.start.x - s.end.x).abs() < 1.0 && (s.start.y - s.end.y).abs() > 1.0
                    });
                    if was_vh {
                        *wire = SchematicWire::manhattan_route_vh_with_net(
                            wire.id,
                            new_start,
                            old_end,
                            wire.net_name.clone(),
                        );
                    } else {
                        *wire = SchematicWire::manhattan_route_hv_with_net(
                            wire.id,
                            new_start,
                            old_end,
                            wire.net_name.clone(),
                        );
                    }
                } else if end_attached {
                    let old_start = wire.start_point();
                    let new_end = wire.end_point() + delta;
                    let was_vh = wire.segments.last().map_or(false, |s| {
                        (s.start.x - s.end.x).abs() < 1.0 && (s.start.y - s.end.y).abs() > 1.0
                    });
                    if was_vh {
                        *wire = SchematicWire::manhattan_route_vh_with_net(
                            wire.id,
                            old_start,
                            new_end,
                            wire.net_name.clone(),
                        );
                    } else {
                        *wire = SchematicWire::manhattan_route_hv_with_net(
                            wire.id,
                            old_start,
                            new_end,
                            wire.net_name.clone(),
                        );
                    }
                }
            }
        }
    }

    /// Renders the rubberband marquee box if active.
    pub fn render_marquee(&self, painter: &Painter) {
        if let (Some(start_world), Some(current_world)) = (self.marquee_start, self.marquee_current) {
            let start_screen = self.world_to_screen(start_world);
            let current_screen = self.world_to_screen(current_world);
            let screen_rect = Rect::from_two_pos(start_screen, current_screen);

            // Semi-transparent fill with theme stroke
            painter.rect_filled(
                screen_rect,
                2.0,
                Color32::from_rgba_unmultiplied(60, 140, 240, 35),
            );
            painter.rect_stroke(
                screen_rect,
                2.0,
                Stroke::new(1.5, Color32::from_rgba_unmultiplied(100, 180, 255, 220)),
                StrokeKind::Outside,
            );
        }
    }

    /// Renders illuminated selection halos around all selected components and wires.
    pub fn render_selection_halos(
        &self,
        painter: &Painter,
        components: &[SchematicComponent],
        wires: &[SchematicWire],
    ) {
        let active_components = if components.is_empty() {
            &self.components[..]
        } else {
            components
        };
        let active_wires = if wires.is_empty() {
            &self.wires[..]
        } else {
            wires
        };

        // Halos for selected components
        for comp in active_components {
            if self.is_component_selected(comp.id) {
                let bbox_world = comp.bounding_box();
                let min_screen = self.world_to_screen(bbox_world.min);
                let max_screen = self.world_to_screen(bbox_world.max);
                let screen_rect = Rect::from_min_max(min_screen, max_screen).expand(4.0 * self.zoom.clamp(0.8, 1.5));

                // Soft glowing background halo
                painter.rect_filled(
                    screen_rect,
                    6.0,
                    Color32::from_rgba_unmultiplied(255, 180, 50, 28),
                );
                // Outer illuminated stroke
                painter.rect_stroke(
                    screen_rect,
                    6.0,
                    Stroke::new(2.0 * self.zoom.clamp(0.8, 1.8), Color32::from_rgba_unmultiplied(255, 190, 60, 200)),
                    StrokeKind::Outside,
                );
            }
        }

        // Halos for selected wires
        for wire in active_wires {
            if self.is_wire_selected(wire.id) {
                let halo_stroke = Stroke::new(
                    6.0 * self.zoom.clamp(0.8, 2.0),
                    Color32::from_rgba_unmultiplied(255, 180, 50, 80),
                );
                for seg in &wire.segments {
                    let s_screen = self.world_to_screen(seg.start);
                    let e_screen = self.world_to_screen(seg.end);
                    painter.line_segment([s_screen, e_screen], halo_stroke);
                }
            }
        }

        // Halos for selected net labels
        for label in &self.net_labels {
            if self.is_label_selected(label.id) {
                let bbox_world = label.bounding_box();
                let min_screen = self.world_to_screen(bbox_world.min);
                let max_screen = self.world_to_screen(bbox_world.max);
                let screen_rect = Rect::from_min_max(min_screen, max_screen).expand(3.0 * self.zoom.clamp(0.8, 1.5));

                painter.rect_filled(
                    screen_rect,
                    4.0,
                    Color32::from_rgba_unmultiplied(255, 180, 50, 35),
                );
                painter.rect_stroke(
                    screen_rect,
                    4.0,
                    Stroke::new(1.8 * self.zoom.clamp(0.8, 1.8), Color32::from_rgba_unmultiplied(255, 190, 60, 210)),
                    StrokeKind::Outside,
                );
            }
        }
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

    /// Draws the background grid (minor dots and major lines) onto the painter using default theme colors.
    pub fn render_grid(&self, painter: &Painter, viewport: Rect) {
        self.render_grid_themed(painter, viewport, &crate::theme::PhononTheme::default());
    }

    /// Draws the background grid using colors from the specified theme.
    pub fn render_grid_themed(&self, painter: &Painter, viewport: Rect, theme: &crate::theme::PhononTheme) {
        if !self.show_grid {
            return;
        }

        let top_left_world = self.screen_to_world(viewport.min);
        let bottom_right_world = self.screen_to_world(viewport.max);

        let start_x = (top_left_world.x / self.grid_size).floor() as i32;
        let end_x = (bottom_right_world.x / self.grid_size).ceil() as i32;
        let start_y = (top_left_world.y / self.grid_size).floor() as i32;
        let end_y = (bottom_right_world.y / self.grid_size).ceil() as i32;

        let dot_color = theme.grid_dot;
        let major_stroke = Stroke::new(1.0, theme.grid_line);

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

    /// Computes the axis-aligned bounding box enclosing all components, wires, subcircuits, and buses in world coordinates.
    pub fn bounding_box(&self) -> Option<Rect> {
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut has_points = false;

        let mut include_pt = |p: Pos2| {
            has_points = true;
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        };

        for comp in &self.components {
            include_pt(comp.pos);
            for (_, p) in comp.all_pins() {
                include_pt(p);
            }
        }

        for wire in &self.wires {
            for seg in &wire.segments {
                include_pt(seg.start);
                include_pt(seg.end);
            }
        }

        for inst in &self.subcircuit_instances {
            include_pt(inst.pos);
            include_pt(Pos2::new(inst.pos.x - 40.0, inst.pos.y - 40.0));
            include_pt(Pos2::new(inst.pos.x + 40.0, inst.pos.y + 40.0));
        }

        for bus in &self.buses {
            for seg in &bus.segments {
                include_pt(seg.start);
                include_pt(seg.end);
            }
        }

        for label in &self.net_labels {
            include_pt(label.pos);
            let b = label.bounding_box();
            include_pt(b.min);
            include_pt(b.max);
        }

        if has_points {
            Some(Rect::from_min_max(
                Pos2::new(min_x, min_y),
                Pos2::new(max_x, max_y),
            ))
        } else {
            None
        }
    }

    /// Automatically centers and fits the canvas contents within the given viewport rect.
    pub fn center_on_bounding_box(&mut self, viewport: Rect) {
        if let Some(bb) = self.bounding_box() {
            let margin = 60.0;
            let avail_w = (viewport.width() - margin * 2.0).max(100.0);
            let avail_h = (viewport.height() - margin * 2.0).max(100.0);

            if bb.width() > 1.0 && bb.height() > 1.0 {
                let zoom_x = avail_w / bb.width();
                let zoom_y = avail_h / bb.height();
                self.zoom = zoom_x.min(zoom_y).clamp(0.4, 2.5);
            }

            let bb_center = bb.center();
            let vp_center = viewport.center();
            self.pan = vp_center.to_vec2() - bb_center.to_vec2() * self.zoom;
        } else {
            self.pan = viewport.center().to_vec2();
            self.zoom = 1.0;
        }
    }
}
