#![deny(unsafe_code)]

//! Vectorized master SVG iconography and pure safe Rust painter engine for Phonon Studio.
//!
//! Provides mathematically rendered emblem widgets depicting a symmetrical Quad Flat Package (QFP)
//! semiconductor integrated circuit with 12 rectangular pins and a contained acoustic sinusoidal waveform.

use egui::{pos2, vec2, Color32, Painter, Rect, Response, Sense, Stroke, StrokeKind, Ui};
use std::f32::consts::TAU;

/// Precision mathematical SVG master icon for Phonon embedded directly in binary.
pub const PHONON_SVG: &str = include_str!("../../../../assets/icons/phonon.svg");

/// Renders the vectorized Phonon master emblem directly onto the UI painter.
///
/// Guarantees a strict 1:1 square aspect ratio (`response.rect.width() == response.rect.height() == size`).
/// Renders:
/// - Pure black sharp square container backing.
/// - Symmetrical Quad Flat Package (QFP) integrated circuit frame with 12 rectangular pins (3 per face).
/// - Contained acoustic sinusoidal waveform centered strictly inside the chip boundary.
pub fn render_phonon_icon(ui: &mut Ui, size: f32) -> Response {
    let size = size.max(1.0);
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::hover());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect);
        paint_phonon_emblem(&painter, rect, size);
    }

    response
}

/// Internal helper rendering the official QFP chip and contained acoustic wave emblem.
fn paint_phonon_emblem(painter: &Painter, rect: Rect, size: f32) {
    let min = rect.min;
    let s = size;

    // 1. Background sharp black square container
    painter.rect_filled(rect, 0.0, Color32::BLACK);

    let pin_color = Color32::WHITE;
    let pin_h = (s * (22.0 / 512.0)).max(1.0);
    let pin_w = (s * (60.0 / 512.0)).max(1.5);
    let chip_stroke_w = (s * (24.0 / 512.0)).max(1.0);

    let chip_x0 = min.x + s * (136.0 / 512.0);
    let chip_y0 = min.y + s * (136.0 / 512.0);
    let chip_size = s * (240.0 / 512.0);

    let y_pins = [
        min.y + s * (186.0 / 512.0),
        min.y + s * (245.0 / 512.0),
        min.y + s * (304.0 / 512.0),
    ];
    let x_pins = [
        min.x + s * (186.0 / 512.0),
        min.x + s * (245.0 / 512.0),
        min.x + s * (304.0 / 512.0),
    ];

    // 2. Left and Right pins
    let left_pin_x = min.x + s * (76.0 / 512.0);
    let right_pin_x = min.x + s * (376.0 / 512.0);
    for &py in &y_pins {
        painter.rect_filled(
            Rect::from_min_size(pos2(left_pin_x, py), vec2(pin_w, pin_h)),
            0.0,
            pin_color,
        );
        painter.rect_filled(
            Rect::from_min_size(pos2(right_pin_x, py), vec2(pin_w, pin_h)),
            0.0,
            pin_color,
        );
    }

    // 3. Top and Bottom pins
    let top_pin_y = min.y + s * (76.0 / 512.0);
    let bot_pin_y = min.y + s * (376.0 / 512.0);
    for &px in &x_pins {
        painter.rect_filled(
            Rect::from_min_size(pos2(px, top_pin_y), vec2(pin_h, pin_w)),
            0.0,
            pin_color,
        );
        painter.rect_filled(
            Rect::from_min_size(pos2(px, bot_pin_y), vec2(pin_h, pin_w)),
            0.0,
            pin_color,
        );
    }

    // 4. Middle square package outline
    let chip_rect = Rect::from_min_size(pos2(chip_x0, chip_y0), vec2(chip_size, chip_size));
    painter.rect_stroke(
        chip_rect,
        0.0,
        Stroke::new(chip_stroke_w, Color32::WHITE),
        StrokeKind::Middle,
    );

    // 5. Contained acoustic wave inside
    let num_pts = 32;
    let wave_start_x = min.x + s * (172.0 / 512.0);
    let wave_end_x = min.x + s * (340.0 / 512.0);
    let wave_w = wave_end_x - wave_start_x;
    let cy = min.y + s * 0.5;
    let amp = s * (60.0 / 512.0);

    let mut wave_pts = Vec::with_capacity(num_pts);
    for i in 0..num_pts {
        let t = i as f32 / (num_pts - 1) as f32;
        let wx = wave_start_x + t * wave_w;
        // Windowed / smooth sine pulse with horizontal lead-ins
        let wy = if t < 0.13 {
            cy
        } else if t > 0.87 {
            cy
        } else {
            let local_t = (t - 0.13) / (0.87 - 0.13);
            cy - (local_t * TAU).sin() * amp
        };
        wave_pts.push(pos2(wx, wy));
    }

    let wave_stroke = Stroke::new((s * (24.0 / 512.0)).max(1.0), Color32::WHITE);
    for pair in wave_pts.windows(2) {
        painter.line_segment([pair[0], pair[1]], wave_stroke);
    }
}
