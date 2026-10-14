#![deny(unsafe_code)]

//! Visual dynamics awareness widgets and discovery status badges for Phonon Studio.
//!
//! Provides the `DynamicsStatusBadge` component which inspects the active physics
//! dynamics backend and renders a visual status badge:
//! - Hardware-accelerated dynamics connected: emerald badge with telemetry tooltip.
//! - Standalone reference dynamics: slate blue badge.

use egui::{Color32, Margin, RichText, Stroke, Ui};
use phonon_core::PhysicsDynamicsBackend;

/// Visual awareness badge indicating the active dynamics backend state in Phonon Studio.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DynamicsStatusBadge;

impl DynamicsStatusBadge {
    /// Creates a new `DynamicsStatusBadge` instance.
    pub fn new() -> Self {
        Self
    }

    /// Evaluates whether the backend corresponds to an accelerated dynamics engine.
    pub fn is_accelerated_mode(&self, backend: &dyn PhysicsDynamicsBackend) -> bool {
        backend.info().is_hardware_accelerated
    }

    /// Returns the badge label string based on active backend capabilities.
    pub fn badge_label(&self, backend: &dyn PhysicsDynamicsBackend) -> &'static str {
        if self.is_accelerated_mode(backend) {
            "[ACTIVE: HARDWARE-ACCELERATED DYNAMICS]"
        } else {
            "[REFERENCE DYNAMICS ACTIVE]"
        }
    }

    /// Returns the badge (background, border) color pair for egui rendering.
    pub fn badge_colors(&self, backend: &dyn PhysicsDynamicsBackend) -> (Color32, Color32) {
        if self.is_accelerated_mode(backend) {
            // Dark emerald/teal with emerald border
            (
                Color32::from_rgb(18, 52, 36),
                Color32::from_rgb(46, 160, 92),
            )
        } else {
            // Dark slate blue with slate blue border
            (
                Color32::from_rgb(26, 40, 56),
                Color32::from_rgb(58, 110, 168),
            )
        }
    }

    /// Formats the detailed hover telemetry tooltip text for the active backend.
    pub fn telemetry_tooltip_text(&self, backend: &dyn PhysicsDynamicsBackend) -> String {
        let info = backend.info();
        let telemetry = backend.current_telemetry();

        format!(
            "Dynamics Backend: {} (v{})\n\
             {}\n\
             Sim Time: {:.3} s | Steps: {}\n\
             Altitude: {:.2} m | Speed: {:.2} m/s\n\
             Position NED: [{:.2}, {:.2}, {:.2}] m\n\
             Velocity NED: [{:.2}, {:.2}, {:.2}] m/s",
            info.name,
            info.version,
            info.description,
            telemetry.sim_time_s,
            telemetry.step_count,
            telemetry.altitude_m(),
            telemetry.speed_m_per_s(),
            telemetry.position_m[0],
            telemetry.position_m[1],
            telemetry.position_m[2],
            telemetry.velocity_m_per_s[0],
            telemetry.velocity_m_per_s[1],
            telemetry.velocity_m_per_s[2]
        )
    }

    /// Renders the visual badge in the given egui UI.
    pub fn ui(&self, ui: &mut Ui, backend: &dyn PhysicsDynamicsBackend) {
        let is_accel = self.is_accelerated_mode(backend);
        let label_text = self.badge_label(backend);
        let (bg_color, border_color) = self.badge_colors(backend);
        let tooltip_text = self.telemetry_tooltip_text(backend);

        ui.horizontal(|ui| {
            let frame = egui::Frame::new()
                .fill(bg_color)
                .stroke(Stroke::new(1.0, border_color))
                .corner_radius(4.0)
                .inner_margin(Margin::symmetric(6, 3));

            let response = frame
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(label_text)
                            .color(if is_accel {
                                Color32::from_rgb(120, 240, 160)
                            } else {
                                Color32::from_rgb(170, 210, 255)
                            })
                            .strong()
                            .monospace()
                            .size(11.0),
                    );
                })
                .response;

            response.on_hover_text(tooltip_text);
        });
    }
}
