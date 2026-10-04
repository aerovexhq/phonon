#![deny(unsafe_code)]

//! Unified high-contrast pill badges and zoom-aware rendering engine for Phonon Studio.

use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2};

/// Visual styling configuration for unified on-canvas pill badges.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PillBadgeStyle {
    pub bg_color: Color32,
    pub border_color: Color32,
    pub text_color: Color32,
    pub border_width: f32,
    pub corner_radius: f32,
}

impl Default for PillBadgeStyle {
    fn default() -> Self {
        Self {
            bg_color: Color32::from_rgba_unmultiplied(18, 24, 34, 230),
            border_color: Color32::from_rgb(60, 160, 240),
            text_color: Color32::from_rgb(220, 240, 255),
            border_width: 1.2,
            corner_radius: 4.0,
        }
    }
}

impl PillBadgeStyle {
    /// Style for node voltage readouts.
    pub fn voltage(color: Color32) -> Self {
        Self {
            bg_color: Color32::from_rgba_unmultiplied(14, 22, 32, 235),
            border_color: color,
            text_color: Color32::from_rgb(200, 240, 255),
            border_width: 1.2,
            corner_radius: 4.0,
        }
    }

    /// Style for junction temperature readouts.
    pub fn temperature(temp_color: Color32, is_hotspot: bool) -> Self {
        if is_hotspot {
            Self {
                bg_color: Color32::from_rgba_unmultiplied(45, 12, 16, 245),
                border_color: Color32::from_rgb(255, 60, 60),
                text_color: Color32::from_rgb(255, 160, 160),
                border_width: 1.8,
                corner_radius: 4.0,
            }
        } else {
            Self {
                bg_color: Color32::from_rgba_unmultiplied(16, 20, 28, 235),
                border_color: temp_color,
                text_color: Color32::from_rgb(240, 245, 250),
                border_width: 1.2,
                corner_radius: 4.0,
            }
        }
    }

    /// Style for wire branch current and telemetry badges.
    pub fn wire_telemetry() -> Self {
        Self {
            bg_color: Color32::from_rgba_unmultiplied(12, 18, 26, 240),
            border_color: Color32::from_rgb(80, 200, 140),
            text_color: Color32::from_rgb(210, 255, 230),
            border_width: 1.4,
            corner_radius: 5.0,
        }
    }

    /// Style for sensitivity / impact badges.
    pub fn sensitivity(badge_color: Color32) -> Self {
        Self {
            bg_color: Color32::from_rgba_unmultiplied(20, 20, 30, 235),
            border_color: badge_color,
            text_color: Color32::WHITE,
            border_width: 1.5,
            corner_radius: 4.0,
        }
    }
}

/// Computes a dampened proportional zoom text and badge scaling factor.
/// Clamps scaling to [0.75, 2.0] so text never collapses into unreadability or balloons.
pub fn proportional_zoom_scale(zoom: f32) -> f32 {
    (zoom.sqrt()).clamp(0.75, 2.0)
}

/// Renders a unified on-canvas pill badge at the given screen coordinates.
/// Returns the bounding screen rectangle occupied by the badge.
pub fn render_pill_badge(
    painter: &Painter,
    center_pos: Pos2,
    text: &str,
    style: &PillBadgeStyle,
    zoom: f32,
) -> Rect {
    let scale = proportional_zoom_scale(zoom);
    let font_size = 10.5 * scale;
    let font = FontId::monospace(font_size);

    // Approximate text width: monospace font width is approx 0.6 * font_size per character
    let text_len = text.chars().count();
    let text_width = (text_len as f32 * font_size * 0.62).max(20.0);
    let text_height = font_size * 1.2;

    let h_padding = 6.0 * scale;
    let v_padding = 3.0 * scale;
    let badge_width = text_width + h_padding * 2.0;
    let badge_height = text_height + v_padding * 2.0;

    let half_size = Vec2::new(badge_width * 0.5, badge_height * 0.5);
    let rect = Rect::from_min_max(center_pos - half_size, center_pos + half_size);

    let rounding = (style.corner_radius * scale).clamp(2.0, badge_height * 0.5);

    // Background pill fill
    painter.rect_filled(rect, rounding, style.bg_color);

    // Border stroke
    painter.rect_stroke(
        rect,
        rounding,
        Stroke::new(style.border_width * scale.clamp(0.8, 1.5), style.border_color),
        StrokeKind::Inside,
    );

    // Monospace text label
    painter.text(
        center_pos,
        Align2::CENTER_CENTER,
        text,
        font,
        style.text_color,
    );

    rect
}

/// Renders a dual-value telemetry pill badge (e.g. `V: 2.50V | I: 2.50mA`).
pub fn render_dual_telemetry_pill(
    painter: &Painter,
    center_pos: Pos2,
    label1: &str,
    val1: &str,
    label2: &str,
    val2: &str,
    style: &PillBadgeStyle,
    zoom: f32,
) -> Rect {
    let combined = format!("{}: {} | {}: {}", label1, val1, label2, val2);
    render_pill_badge(painter, center_pos, &combined, style, zoom)
}
