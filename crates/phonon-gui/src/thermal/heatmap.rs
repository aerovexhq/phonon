#![deny(unsafe_code)]

//! 2D spatial thermal heatmaps, colormap gradients (Turbo, Magma, Inferno), and hotspot alerts.

use egui::{Align2, Color32, FontId, Painter, Pos2, Vec2};

/// Colormaps for scientific thermal visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Colormap {
    #[default]
    Turbo,
    Magma,
    Inferno,
}

/// Maps a normalized scalar value $u \in [0.0, 1.0]$ to an RGB color.
pub fn sample_colormap(u: f32, map: Colormap) -> Color32 {
    let t = u.clamp(0.0, 1.0);
    match map {
        Colormap::Turbo => {
            // Polynomial approximation of Google Turbo colormap
            let r = (34.61 + t * (1172.0 + t * (1072.0 - t * (1434.0 + t * (2313.0 - t * 1038.0)))))
                .clamp(0.0, 255.0) as u8;
            let g = (23.31 + t * (557.3 + t * (1074.0 - t * (2457.0 - t * (1228.0 - t * 545.0)))))
                .clamp(0.0, 255.0) as u8;
            let b = (27.2
                + t * (3211.0 - t * (15327.0 - t * (27814.0 - t * (22569.0 - t * 6832.0)))))
                .clamp(0.0, 255.0) as u8;
            Color32::from_rgb(r, g, b)
        }
        Colormap::Magma => {
            // Smooth Magma gradient
            let r = (255.0 * (1.1 * t).clamp(0.0, 1.0)) as u8;
            let g = (255.0 * (t.powf(1.8) * 0.9).clamp(0.0, 1.0)) as u8;
            let b = (255.0 * (0.3 + 0.7 * (1.0 - t).powi(2)).clamp(0.0, 1.0)) as u8;
            Color32::from_rgb(r, g, b)
        }
        Colormap::Inferno => {
            // Smooth Inferno gradient
            let r = (255.0 * (t * 1.2).clamp(0.0, 1.0)) as u8;
            let g = (255.0 * (t.powf(2.0)).clamp(0.0, 1.0)) as u8;
            let b = (255.0 * (0.1 + 0.4 * (1.0 - (t - 0.5).abs() * 2.0).clamp(0.0, 1.0))) as u8;
            Color32::from_rgb(r, g, b)
        }
    }
}

/// Thermal visualization manager.
#[derive(Debug, Clone)]
pub struct ThermalOverlay {
    pub enabled: bool,
    pub colormap: Colormap,
    pub min_temp_c: f64,
    pub max_temp_c: f64,
    pub runaway_threshold_c: f64,
}

impl Default for ThermalOverlay {
    fn default() -> Self {
        Self {
            enabled: true,
            colormap: Colormap::Turbo,
            min_temp_c: 25.0,
            max_temp_c: 125.0,
            runaway_threshold_c: 120.0,
        }
    }
}

impl ThermalOverlay {
    pub fn new() -> Self {
        Self::default()
    }

    /// Maps a physical temperature in Celsius to an RGB color.
    pub fn temp_to_color(&self, temp_c: f64) -> Color32 {
        let span = (self.max_temp_c - self.min_temp_c).max(1.0);
        let norm = ((temp_c - self.min_temp_c) / span) as f32;
        sample_colormap(norm, self.colormap)
    }

    /// Renders a thermal junction badge near a component scaled with canvas zoom.
    pub fn render_junction_badge_scaled(
        &self,
        painter: &Painter,
        pos_screen: Pos2,
        temp_c: f64,
        component_name: &str,
        zoom: f32,
    ) {
        if !self.enabled {
            return;
        }

        let is_hotspot = temp_c >= self.runaway_threshold_c;
        let badge_color = self.temp_to_color(temp_c);
        let style = crate::widgets::pill_badge::PillBadgeStyle::temperature(badge_color, is_hotspot);

        let text = if is_hotspot {
            format!("{:.1}°C [HOT]", temp_c)
        } else {
            format!("{:.1}°C", temp_c)
        };

        let badge_rect = crate::widgets::pill_badge::render_pill_badge(
            painter,
            pos_screen,
            &text,
            &style,
            zoom,
        );

        // Warning indicator if exceeding safe thermal limits
        if is_hotspot {
            let scale = crate::widgets::pill_badge::proportional_zoom_scale(zoom);
            let icon_pos = badge_rect.right_top() + Vec2::new(4.0 * scale, 1.0 * scale);
            painter.text(
                icon_pos,
                Align2::LEFT_TOP,
                format!("[WARN] {} HOT", component_name),
                FontId::proportional(9.5 * scale),
                Color32::from_rgb(255, 80, 80),
            );
        }
    }

    /// Renders a thermal junction badge near a component.
    pub fn render_junction_badge(
        &self,
        painter: &Painter,
        pos_screen: Pos2,
        temp_c: f64,
        component_name: &str,
    ) {
        self.render_junction_badge_scaled(painter, pos_screen, temp_c, component_name, 1.0);
    }
}
