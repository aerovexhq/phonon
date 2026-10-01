#![deny(unsafe_code)]

//! Vectorized master SVG iconography and pure safe Rust painter engine for Phonon Studio.
//!
//! Provides mathematically rendered emblem widgets depicting acoustic phonon wavepackets
//! traversing a 2D semiconductor crystal lattice with anti-aliased quantum displacement waves.

use egui::{pos2, vec2, Color32, Painter, Rect, Response, Sense, Stroke, StrokeKind, Ui};
use std::f32::consts::TAU;

/// Precision mathematical SVG master icon for Phonon embedded directly in binary.
pub const PHONON_SVG: &str = include_str!("../../../../assets/icons/phonon.svg");

/// Renders the vectorized Phonon master emblem directly onto the UI painter.
///
/// Guarantees a strict 1:1 square aspect ratio (`response.rect.width() == response.rect.height() == size`).
/// Renders:
/// - Rounded dark container backing with border.
/// - 2D semiconductor crystal lattice grid lines.
/// - Concentric quantum acoustic circular wavefronts.
/// - Anti-aliased sinusoidal acoustic displacement wavepacket curves with Gaussian envelopes.
/// - Semiconductor crystal lattice atomic nodes.
/// - Central acoustic energy wavepacket core.
pub fn render_phonon_icon(ui: &mut Ui, size: f32) -> Response {
    let size = size.max(1.0);
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::hover());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect);
        paint_phonon_emblem(&painter, rect, size);
    }

    response
}

/// Internal helper rendering the multi-scale phonon emblem elements into the painter.
fn paint_phonon_emblem(painter: &Painter, rect: Rect, size: f32) {
    let center = rect.center();
    let radius = size * 0.5;

    // 1. Background rounded emblem container
    let bg_color = Color32::from_rgb(11, 15, 25);
    let border_color = Color32::from_rgb(30, 41, 59);
    let corner_radius = (size * 0.2).clamp(2.0, 16.0);
    painter.rect_filled(rect, corner_radius, bg_color);
    painter.rect_stroke(
        rect,
        corner_radius,
        Stroke::new(1.0, border_color),
        StrokeKind::Inside,
    );

    // 2. 2D Semiconductor crystal lattice grid (subtle background orthogonal lines)
    let grid_stroke = Stroke::new(
        0.8,
        Color32::from_rgba_unmultiplied(51, 65, 85, 110),
    );
    let step = size * 0.18;
    for i in -2..=2 {
        let offset = i as f32 * step;
        // Horizontal lattice lines
        painter.line_segment(
            [
                pos2(center.x - radius * 0.72, center.y + offset),
                pos2(center.x + radius * 0.72, center.y + offset),
            ],
            grid_stroke,
        );
        // Vertical lattice lines
        painter.line_segment(
            [
                pos2(center.x + offset, center.y - radius * 0.72),
                pos2(center.x + offset, center.y + radius * 0.72),
            ],
            grid_stroke,
        );
    }

    // 3. Concentric phonon acoustic wavefronts
    let ring_color1 = Color32::from_rgba_unmultiplied(6, 182, 212, 60);
    let ring_color2 = Color32::from_rgba_unmultiplied(56, 189, 248, 90);
    let ring_color3 = Color32::from_rgba_unmultiplied(129, 140, 248, 120);

    painter.circle_stroke(center, radius * 0.70, Stroke::new(1.0, ring_color1));
    painter.circle_stroke(center, radius * 0.52, Stroke::new(1.2, ring_color2));
    painter.circle_stroke(center, radius * 0.35, Stroke::new(1.5, ring_color3));

    // 4. Primary quantum acoustic sinusoidal wavepacket (Gaussian enveloped)
    let num_samples = 32;
    let wave_width = size * 0.76;
    let start_x = center.x - wave_width * 0.5;
    let k = TAU * 2.0 / wave_width;
    let amp = size * 0.16;

    let mut points = Vec::with_capacity(num_samples);
    for s in 0..num_samples {
        let frac = s as f32 / (num_samples - 1) as f32;
        let x = start_x + frac * wave_width;
        let rel_x = x - center.x;
        // Gaussian wavepacket envelope
        let envelope = (-((rel_x / (wave_width * 0.36)).powi(2))).exp();
        let y = center.y + (rel_x * k).sin() * amp * envelope;
        points.push(pos2(x, y));
    }

    let wave_stroke = Stroke::new(
        (size * 0.07).clamp(1.2, 2.5),
        Color32::from_rgb(56, 189, 248),
    );
    for pair in points.windows(2) {
        painter.line_segment([pair[0], pair[1]], wave_stroke);
    }

    // 5. Secondary anti-phase harmonic wavepacket
    let mut points2 = Vec::with_capacity(num_samples);
    for s in 0..num_samples {
        let frac = s as f32 / (num_samples - 1) as f32;
        let x = start_x + frac * wave_width;
        let rel_x = x - center.x;
        let envelope = (-((rel_x / (wave_width * 0.42)).powi(2))).exp();
        let y = center.y - (rel_x * k).sin() * (amp * 0.55) * envelope;
        points2.push(pos2(x, y));
    }
    let wave_stroke2 = Stroke::new(
        (size * 0.05).clamp(0.8, 1.6),
        Color32::from_rgba_unmultiplied(168, 85, 247, 180),
    );
    for pair in points2.windows(2) {
        painter.line_segment([pair[0], pair[1]], wave_stroke2);
    }

    // 6. Crystal lattice atomic nodes (discrete semiconductor dots)
    let node_radius = (size * 0.035).clamp(1.2, 3.0);
    let atom_color = Color32::from_rgb(129, 140, 248);
    for i in [-1.0, 0.0, 1.0] {
        for j in [-1.0, 0.0, 1.0] {
            if i != 0.0 || j != 0.0 {
                let node_pos = pos2(center.x + i * step, center.y + j * step);
                painter.circle_filled(node_pos, node_radius, atom_color);
            }
        }
    }

    // 7. Central acoustic energy wavepacket core
    let core_radius = (size * 0.12).clamp(2.5, 6.0);
    painter.circle_filled(center, core_radius, Color32::from_rgb(15, 23, 42));
    painter.circle_stroke(
        center,
        core_radius,
        Stroke::new(1.5, Color32::from_rgb(56, 189, 248)),
    );
    painter.circle_filled(center, core_radius * 0.45, Color32::from_rgb(56, 189, 248));
}
