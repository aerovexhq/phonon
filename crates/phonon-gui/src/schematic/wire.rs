#![deny(unsafe_code)]

//! Orthogonal Manhattan wire routing, segment management, and junction detection.

use super::canvas::SchematicCanvas;
use egui::{Painter, Pos2, Rect, Stroke};

/// Pin normal or departure orientation for pin-aware Manhattan routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinNormal {
    North,
    South,
    East,
    West,
    Vertical,
    Horizontal,
}

impl PinNormal {
    pub fn is_vertical(&self) -> bool {
        matches!(self, Self::North | Self::South | Self::Vertical)
    }

    pub fn is_horizontal(&self) -> bool {
        matches!(self, Self::East | Self::West | Self::Horizontal)
    }

    pub fn from_vec(v: egui::Vec2) -> Self {
        if v.y.abs() >= v.x.abs() {
            if v.y <= 0.0 {
                Self::North
            } else {
                Self::South
            }
        } else {
            if v.x >= 0.0 {
                Self::East
            } else {
                Self::West
            }
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
            Self::Vertical => Self::Vertical,
            Self::Horizontal => Self::Horizontal,
        }
    }
}

pub type WirePinOrientation = PinNormal;

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

    /// Checks if this wire segment intersects or is contained within the given rectangle in world coordinates.
    pub fn intersects_rect(&self, rect: &Rect) -> bool {
        if rect.contains(self.start) || rect.contains(self.end) {
            return true;
        }
        let p_tl = rect.min;
        let p_tr = Pos2::new(rect.max.x, rect.min.y);
        let p_br = rect.max;
        let p_bl = Pos2::new(rect.min.x, rect.max.y);

        segments_intersect(self.start, self.end, p_tl, p_tr)
            || segments_intersect(self.start, self.end, p_tr, p_br)
            || segments_intersect(self.start, self.end, p_br, p_bl)
            || segments_intersect(self.start, self.end, p_bl, p_tl)
    }
}

/// A multi-segment electrical connection wire.
#[derive(Debug, Clone, PartialEq)]
pub struct SchematicWire {
    pub id: usize,
    pub segments: Vec<WireSegment>,
    pub net_name: Option<String>,
    /// Number of bits carried by this wire (default 1). Values > 1 indicate a compressed bus.
    pub bit_width: u32,
}

impl SchematicWire {
    pub fn new(id: usize, segments: Vec<WireSegment>) -> Self {
        Self {
            id,
            segments,
            net_name: None,
            bit_width: 1,
        }
    }

    /// Builder method attaching an optional or explicit net name.
    pub fn with_net_name(mut self, net_name: impl Into<String>) -> Self {
        self.net_name = Some(net_name.into());
        self
    }

    /// Builder method configuring the bus bit width.
    pub fn with_bit_width(mut self, width: u32) -> Self {
        self.bit_width = width.max(1);
        self
    }

    /// Returns the bit width of this wire (1 for single wire, >1 for compressed bus).
    pub fn bit_width(&self) -> u32 {
        self.bit_width
    }

    /// Returns true if this wire is a multi-bit compressed bus (bit_width > 1).
    pub fn is_bus(&self) -> bool {
        self.bit_width > 1
    }

    /// Returns the start point of the first wire segment, or (0, 0) if empty.
    pub fn start_point(&self) -> Pos2 {
        self.segments.first().map(|s| s.start).unwrap_or(Pos2::ZERO)
    }

    /// Returns the end point of the last wire segment, or (0, 0) if empty.
    pub fn end_point(&self) -> Pos2 {
        self.segments.last().map(|s| s.end).unwrap_or(Pos2::ZERO)
    }

    /// Creates an orthogonal Manhattan route: Horizontal first, then Vertical.
    /// Corner is at `Pos2::new(to.x, from.y)`.
    pub fn manhattan_route_hv(id: usize, from: Pos2, to: Pos2) -> Self {
        let mut segments = Vec::new();
        if (from.x - to.x).abs() > 0.1 && (from.y - to.y).abs() > 0.1 {
            let corner = Pos2::new(to.x, from.y);
            segments.push(WireSegment::new(from, corner));
            segments.push(WireSegment::new(corner, to));
        } else if (from.x - to.x).abs() > 0.1 || (from.y - to.y).abs() > 0.1 {
            segments.push(WireSegment::new(from, to));
        }
        Self {
            id,
            segments,
            net_name: None,
            bit_width: 1,
        }
    }

    /// Creates an orthogonal Manhattan route: Vertical first, then Horizontal.
    /// Corner is at `Pos2::new(from.x, to.y)`.
    pub fn manhattan_route_vh(id: usize, from: Pos2, to: Pos2) -> Self {
        let mut segments = Vec::new();
        if (from.x - to.x).abs() > 0.1 && (from.y - to.y).abs() > 0.1 {
            let corner = Pos2::new(from.x, to.y);
            segments.push(WireSegment::new(from, corner));
            segments.push(WireSegment::new(corner, to));
        } else if (from.x - to.x).abs() > 0.1 || (from.y - to.y).abs() > 0.1 {
            segments.push(WireSegment::new(from, to));
        }
        Self {
            id,
            segments,
            net_name: None,
            bit_width: 1,
        }
    }

    /// Creates an orthogonal Manhattan route between two points (horizontal then vertical).
    pub fn manhattan_route(id: usize, from: Pos2, to: Pos2) -> Self {
        Self::manhattan_route_hv(id, from, to)
    }

    /// Creates an orthogonal Manhattan route with an explicit net name (horizontal then vertical).
    pub fn manhattan_route_with_net(
        id: usize,
        from: Pos2,
        to: Pos2,
        net_name: Option<String>,
    ) -> Self {
        Self::manhattan_route_hv_with_net(id, from, to, net_name)
    }

    /// Creates an orthogonal Manhattan route (HV) with an explicit net name.
    pub fn manhattan_route_hv_with_net(
        id: usize,
        from: Pos2,
        to: Pos2,
        net_name: Option<String>,
    ) -> Self {
        let mut w = Self::manhattan_route_hv(id, from, to);
        w.net_name = net_name;
        w
    }

    /// Creates an orthogonal Manhattan route (VH) with an explicit net name.
    pub fn manhattan_route_vh_with_net(
        id: usize,
        from: Pos2,
        to: Pos2,
        net_name: Option<String>,
    ) -> Self {
        let mut w = Self::manhattan_route_vh(id, from, to);
        w.net_name = net_name;
        w
    }

    /// Creates a Manhattan route aware of the departure pin's normal or orientation.
    /// If the pin exits vertically (North/South), routes VH (vertical first).
    /// If the pin exits horizontally (East/West), routes HV (horizontal first).
    pub fn manhattan_route_pin_aware(
        id: usize,
        from: Pos2,
        from_normal: PinNormal,
        to: Pos2,
    ) -> Self {
        if from_normal.is_vertical() {
            Self::manhattan_route_vh(id, from, to)
        } else {
            Self::manhattan_route_hv(id, from, to)
        }
    }

    /// Creates a Manhattan route aware of the departure pin's normal or orientation, with an optional net name.
    pub fn manhattan_route_pin_aware_with_net(
        id: usize,
        from: Pos2,
        from_normal: PinNormal,
        to: Pos2,
        net_name: Option<String>,
    ) -> Self {
        let mut w = Self::manhattan_route_pin_aware(id, from, from_normal, to);
        w.net_name = net_name;
        w
    }

    /// Checks if this wire intersects any obstacle in the given slice.
    /// Obstacle rectangles are shrunk by 2.0px to avoid false positives along grid boundary channels.
    pub fn intersects_obstacles(&self, obstacles: &[Rect]) -> bool {
        for seg in &self.segments {
            for obs in obstacles {
                if obs.width() > 4.0 && obs.height() > 4.0 {
                    if seg.intersects_rect(&obs.shrink(2.0)) {
                        return true;
                    }
                } else if seg.intersects_rect(obs) {
                    return true;
                }
            }
        }
        false
    }

    /// Creates an obstacle-avoiding Manhattan route between `from` and `to`.
    /// Respects the departure pin normal `from_normal` and arrival pin normal `to_normal`.
    /// Tests candidate orthogonal paths against `obstacles`. Returns `Some(wire)` if a clean path
    /// is found, or `None` if all candidates intersect obstacles.
    pub fn manhattan_route_avoiding_obstacles(
        id: usize,
        from: Pos2,
        from_normal: PinNormal,
        to: Pos2,
        to_normal: Option<PinNormal>,
        obstacles: &[Rect],
        net_name: Option<String>,
    ) -> Option<Self> {
        let mut candidates = Vec::new();

        // 1. Direct pin-normal aware 2-segment path
        if from_normal.is_vertical() {
            candidates.push(vec![from, Pos2::new(from.x, to.y), to]);
        } else {
            candidates.push(vec![from, Pos2::new(to.x, from.y), to]);
        }

        // 2. Direct pin-normal arrival 2-segment path
        if let Some(tn) = to_normal {
            if tn.is_vertical() {
                candidates.push(vec![from, Pos2::new(to.x, from.y), to]);
            } else {
                candidates.push(vec![from, Pos2::new(from.x, to.y), to]);
            }
        }

        // 3. Alternative 2-segment path
        if from_normal.is_vertical() {
            candidates.push(vec![from, Pos2::new(to.x, from.y), to]);
        } else {
            candidates.push(vec![from, Pos2::new(from.x, to.y), to]);
        }

        // 4. Midpoint Z-channels
        let mid_y = ((from.y + to.y) * 0.5 / 20.0).round() * 20.0;
        candidates.push(vec![from, Pos2::new(from.x, mid_y), Pos2::new(to.x, mid_y), to]);

        let mid_x = ((from.x + to.x) * 0.5 / 20.0).round() * 20.0;
        candidates.push(vec![from, Pos2::new(mid_x, from.y), Pos2::new(mid_x, to.y), to]);

        // 5. Clearance channels around obstacles
        for obs in obstacles {
            let y_top = (obs.min.y / 20.0).floor() * 20.0 - 20.0;
            candidates.push(vec![from, Pos2::new(from.x, y_top), Pos2::new(to.x, y_top), to]);

            let y_bot = (obs.max.y / 20.0).ceil() * 20.0 + 20.0;
            candidates.push(vec![from, Pos2::new(from.x, y_bot), Pos2::new(to.x, y_bot), to]);

            let x_left = (obs.min.x / 20.0).floor() * 20.0 - 20.0;
            candidates.push(vec![from, Pos2::new(x_left, from.y), Pos2::new(x_left, to.y), to]);

            let x_right = (obs.max.x / 20.0).ceil() * 20.0 + 20.0;
            candidates.push(vec![from, Pos2::new(x_right, from.y), Pos2::new(x_right, to.y), to]);
        }

        for cand_pts in candidates {
            let wire = wire_from_points(id, &cand_pts, net_name.clone());
            if !wire.intersects_obstacles(obstacles) {
                return Some(wire);
            }
        }

        None
    }

    /// Checks if any segment of this wire intersects or is contained within the given rectangle.
    pub fn intersects_rect(&self, rect: &Rect) -> bool {
        self.segments.iter().any(|seg| seg.intersects_rect(rect))
    }

    /// Computes the axis-aligned bounding box enclosing all wire segments.
    pub fn bounding_box(&self) -> Rect {
        if self.segments.is_empty() {
            return Rect::from_min_max(Pos2::ZERO, Pos2::ZERO);
        }
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for seg in &self.segments {
            min_x = min_x.min(seg.start.x).min(seg.end.x);
            min_y = min_y.min(seg.start.y).min(seg.end.y);
            max_x = max_x.max(seg.start.x).max(seg.end.x);
            max_y = max_y.max(seg.start.y).max(seg.end.y);
        }
        Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y))
    }

    /// Hit testing for wire selection.
    pub fn contains(&self, world_pos: Pos2, tol: f32) -> bool {
        self.segments
            .iter()
            .any(|seg| seg.contains_point(world_pos, tol))
    }

    /// Renders the wire onto the painter with default theme colors.
    pub fn render(&self, painter: &Painter, canvas: &SchematicCanvas, is_selected: bool) {
        self.render_with_theme(painter, canvas, is_selected, &crate::theme::PhononTheme::default());
    }

    /// Renders the wire onto the painter using customizable theme colors.
    pub fn render_with_theme(
        &self,
        painter: &Painter,
        canvas: &SchematicCanvas,
        is_selected: bool,
        theme: &crate::theme::PhononTheme,
    ) {
        let color = if is_selected {
            theme.wire_selected
        } else {
            theme.wire_normal
        };
        let stroke_width = if self.bit_width > 1 {
            (3.5 * canvas.zoom).clamp(2.5, 5.0)
        } else {
            2.0 * canvas.zoom.clamp(0.8, 2.0)
        };
        let stroke = Stroke::new(stroke_width, color);

        for seg in &self.segments {
            let s_screen = canvas.world_to_screen(seg.start);
            let e_screen = canvas.world_to_screen(seg.end);
            painter.line_segment([s_screen, e_screen], stroke);
        }

        // Draw Logisim-style bus slash & width badge "/N" at the midpoint of the longest segment
        if self.bit_width > 1 && !self.segments.is_empty() {
            if let Some(longest) = self
                .segments
                .iter()
                .max_by(|a, b| a.length().partial_cmp(&b.length()).unwrap_or(std::cmp::Ordering::Equal))
            {
                if longest.length() > 10.0 {
                    let mid_world = Pos2::new(
                        (longest.start.x + longest.end.x) * 0.5,
                        (longest.start.y + longest.end.y) * 0.5,
                    );
                    let mid_screen = canvas.world_to_screen(mid_world);
                    let slash_vec = egui::Vec2::new(-4.0, -7.0) * canvas.zoom.clamp(0.8, 1.5);
                    let p1 = mid_screen + slash_vec;
                    let p2 = mid_screen - slash_vec;
                    let slash_stroke = Stroke::new((2.0 * canvas.zoom).clamp(1.5, 3.0), color);
                    painter.line_segment([p1, p2], slash_stroke);

                    let label = format!("/{}", self.bit_width);
                    let font_id = egui::FontId::monospace(10.0 * canvas.zoom.clamp(0.8, 1.4));
                    let text_pos =
                        mid_screen + egui::Vec2::new(5.0, -10.0) * canvas.zoom.clamp(0.8, 1.4);
                    painter.text(
                        text_pos,
                        egui::Align2::LEFT_BOTTOM,
                        label,
                        font_id,
                        color,
                    );
                }
            }
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

/// Checks if two 2D line segments (p1-p2 and p3-p4) intersect.
pub fn segments_intersect(a: Pos2, b: Pos2, c: Pos2, d: Pos2) -> bool {
    fn ccw(p1: Pos2, p2: Pos2, p3: Pos2) -> f32 {
        (p2.x - p1.x) * (p3.y - p1.y) - (p2.y - p1.y) * (p3.x - p1.x)
    }

    let d1 = ccw(c, d, a);
    let d2 = ccw(c, d, b);
    let d3 = ccw(a, b, c);
    let d4 = ccw(a, b, d);

    if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
    {
        return true;
    }

    let on_segment = |p: Pos2, q: Pos2, r: Pos2| -> bool {
        q.x >= p.x.min(r.x) - 1e-4
            && q.x <= p.x.max(r.x) + 1e-4
            && q.y >= p.y.min(r.y) - 1e-4
            && q.y <= p.y.max(r.y) + 1e-4
    };

    if d1.abs() < 1e-5 && on_segment(c, a, d) {
        return true;
    }
    if d2.abs() < 1e-5 && on_segment(c, b, d) {
        return true;
    }
    if d3.abs() < 1e-5 && on_segment(a, c, b) {
        return true;
    }
    if d4.abs() < 1e-5 && on_segment(a, d, b) {
        return true;
    }

    false
}

/// Constructs a `SchematicWire` from a sequence of orthogonal points,
/// filtering out zero-length segments and merging contiguous collinear segments.
pub fn wire_from_points(id: usize, points: &[Pos2], net_name: Option<String>) -> SchematicWire {
    let mut segments: Vec<WireSegment> = Vec::new();
    for window in points.windows(2) {
        let p1 = window[0];
        let p2 = window[1];
        if (p1.x - p2.x).abs() > 0.1 || (p1.y - p2.y).abs() > 0.1 {
            if let Some(last) = segments.last_mut() {
                let collinear_h = (last.start.y - last.end.y).abs() < 0.1
                    && (p1.y - p2.y).abs() < 0.1
                    && (last.end.y - p1.y).abs() < 0.1;
                let collinear_v = (last.start.x - last.end.x).abs() < 0.1
                    && (p1.x - p2.x).abs() < 0.1
                    && (last.end.x - p1.x).abs() < 0.1;
                if collinear_h || collinear_v {
                    last.end = p2;
                    continue;
                }
            }
            segments.push(WireSegment::new(p1, p2));
        }
    }
    SchematicWire {
        id,
        segments,
        net_name,
        bit_width: 1,
    }
}

/// Represents an orthogonal intersection between two wires where they cross without an electrical connection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WireCrossing {
    /// World position of the crossing intersection.
    pub point: Pos2,
    /// Whether the bridge arc is along the horizontal segment.
    pub is_horizontal: bool,
}

/// Identifies all geometric wire crossings that do NOT have a solder junction dot.
/// These are orthogonal crossing paths that must render a visual bridge hop.
pub fn compute_wire_crossings(wires: &[SchematicWire], junctions: &[Pos2]) -> Vec<WireCrossing> {
    let mut crossings = Vec::new();
    let tol = 3.0;

    for (w1_idx, w1) in wires.iter().enumerate() {
        for seg1 in &w1.segments {
            let seg1_h = (seg1.start.y - seg1.end.y).abs() <= 0.1;
            let seg1_v = (seg1.start.x - seg1.end.x).abs() <= 0.1;

            for w2 in wires.iter().skip(w1_idx) {
                for seg2 in &w2.segments {
                    if std::ptr::eq(seg1, seg2) {
                        continue;
                    }

                    let seg2_h = (seg2.start.y - seg2.end.y).abs() <= 0.1;
                    let seg2_v = (seg2.start.x - seg2.end.x).abs() <= 0.1;

                    // Perpendicular crossings (H crosses V)
                    if (seg1_h && seg2_v) || (seg1_v && seg2_h) {
                        let (h_seg, v_seg) = if seg1_h {
                            (seg1, seg2)
                        } else {
                            (seg2, seg1)
                        };

                        let cross_x = v_seg.start.x;
                        let cross_y = h_seg.start.y;
                        let pt = Pos2::new(cross_x, cross_y);

                        let h_min_x = h_seg.start.x.min(h_seg.end.x);
                        let h_max_x = h_seg.start.x.max(h_seg.end.x);
                        let v_min_y = v_seg.start.y.min(v_seg.end.y);
                        let v_max_y = v_seg.start.y.max(v_seg.end.y);

                        // Must be strictly interior (not at endpoints)
                        if cross_x > h_min_x + tol
                            && cross_x < h_max_x - tol
                            && cross_y > v_min_y + tol
                            && cross_y < v_max_y - tol
                        {
                            // Ensure there is no solder junction dot at this point
                            let has_junction = junctions.iter().any(|&j| (j - pt).length() <= tol);
                            if !has_junction && !crossings.iter().any(|c: &WireCrossing| (c.point - pt).length() <= tol) {
                                crossings.push(WireCrossing {
                                    point: pt,
                                    is_horizontal: true,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    crossings
}

/// Renders visual bridge hop arcs for wire crossings onto the canvas painter.
pub fn render_wire_crossings(
    painter: &Painter,
    canvas: &SchematicCanvas,
    crossings: &[WireCrossing],
    theme: &crate::theme::PhononTheme,
) {
    let wire_color = theme.wire_normal;
    let bg_color = theme.canvas_bg;
    let zoom = canvas.zoom.clamp(0.6, 2.5);
    let hop_radius = 4.5 * zoom;
    let stroke_width = 2.0 * zoom;

    for crossing in crossings {
        let center_screen = canvas.world_to_screen(crossing.point);

        // 1. Mask out the vertical wire underneath with background color
        let mask_rect = Rect::from_center_size(center_screen, egui::vec2(hop_radius * 2.2, hop_radius * 2.0));
        painter.rect_filled(mask_rect, 0.0, bg_color);

        // 2. Draw bridge hop arc on the horizontal wire
        let left_pt = Pos2::new(center_screen.x - hop_radius, center_screen.y);
        let right_pt = Pos2::new(center_screen.x + hop_radius, center_screen.y);
        let p_top1 = Pos2::new(center_screen.x - hop_radius * 0.6, center_screen.y - hop_radius * 0.85);
        let p_peak = Pos2::new(center_screen.x, center_screen.y - hop_radius);
        let p_top2 = Pos2::new(center_screen.x + hop_radius * 0.6, center_screen.y - hop_radius * 0.85);

        let stroke = Stroke::new(stroke_width, wire_color);
        painter.line_segment([left_pt, p_top1], stroke);
        painter.line_segment([p_top1, p_peak], stroke);
        painter.line_segment([p_peak, p_top2], stroke);
        painter.line_segment([p_top2, right_pt], stroke);

        // 3. Draw vertical wire segments approaching the bridge
        let v_top = Pos2::new(center_screen.x, mask_rect.min.y);
        let v_bot = Pos2::new(center_screen.x, mask_rect.max.y);
        painter.line_segment([Pos2::new(center_screen.x, center_screen.y - hop_radius * 1.5), v_top], stroke);
        painter.line_segment([v_bot, Pos2::new(center_screen.x, center_screen.y + hop_radius * 1.5)], stroke);
    }
}



