#![deny(unsafe_code)]

//! Interactive 3D Physical Card Viewport with Abstracted Ground Plane.
//!
//! Provides a real-time 3D perspective visualization of the synthesized physical
//! circuit board with orbit controls, abstracted ground grid, soft drop shadows,
//! realistic extruded DIP and SMD packages, copper traces, and EMI disturbance telemetry.

use crate::board_synthesis::card_model::{Face3D, SolderMaskColor, SynthesizedPhysicalCard};
use crate::board_synthesis::trace_router::CopperLayer;
use crate::board_synthesis::{AutoPlacementParams, BoardSynthesisEngine, SynthesisError, TechMappingOptions};
use crate::schematic::components::SchematicComponent;
use crate::schematic::wire::SchematicWire;
use egui::{Color32, Pos2, RichText, Sense, Stroke, Ui, Vec2};

/// Interactive 3D camera parameters for orbiting the board.
#[derive(Debug, Clone, PartialEq)]
pub struct Camera3D {
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub zoom: f32,
    pub pan: Vec2,
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            yaw_deg: 35.0,
            pitch_deg: 38.0,
            zoom: 8.5,
            pan: Vec2::ZERO,
        }
    }
}

/// State and UI controller for the 3D Physical Card Viewport.
#[derive(Debug, Clone)]
pub struct Card3dViewport {
    pub camera: Camera3D,
    pub synthesized_card: Option<SynthesizedPhysicalCard>,
    pub tech_options: TechMappingOptions,
    pub placement_params: AutoPlacementParams,
    pub solder_mask_selection: SolderMaskColor,
    pub show_abstracted_ground: bool,
    pub show_copper_traces: bool,
    pub show_shadow: bool,
    pub last_synthesis_error: Option<String>,
}

impl Default for Card3dViewport {
    fn default() -> Self {
        Self {
            camera: Camera3D::default(),
            synthesized_card: None,
            tech_options: TechMappingOptions::default(),
            placement_params: AutoPlacementParams::default(),
            solder_mask_selection: SolderMaskColor::ObsidianMatteBlack,
            show_abstracted_ground: true,
            show_copper_traces: true,
            show_shadow: true,
            last_synthesis_error: None,
        }
    }
}

impl Card3dViewport {
    /// Resets camera to standard perspective angle.
    pub fn reset_camera_perspective(&mut self) {
        self.camera.yaw_deg = 35.0;
        self.camera.pitch_deg = 38.0;
        self.camera.pan = Vec2::ZERO;
        self.camera.zoom = 8.5;
    }

    /// Resets camera to top-down 2D orthographic view.
    pub fn reset_camera_top(&mut self) {
        self.camera.yaw_deg = 0.0;
        self.camera.pitch_deg = 89.9;
        self.camera.pan = Vec2::ZERO;
    }

    /// Resets camera to isometric view.
    pub fn reset_camera_isometric(&mut self) {
        self.camera.yaw_deg = 45.0;
        self.camera.pitch_deg = 30.0;
        self.camera.pan = Vec2::ZERO;
    }

    /// Triggers synthesis of the physical board card from current schematic state.
    pub fn run_synthesis(
        &mut self,
        components: &[SchematicComponent],
        wires: &[SchematicWire],
    ) -> Result<(), SynthesisError> {
        let mut card = BoardSynthesisEngine::synthesize_card(
            components,
            wires,
            &self.tech_options,
            &self.placement_params,
        )?;
        card.solder_mask = self.solder_mask_selection;
        self.synthesized_card = Some(card);
        self.last_synthesis_error = None;
        Ok(())
    }

    /// Main render entry point for the 3D card viewport.
    pub fn render(
        &mut self,
        ui: &mut Ui,
        components: &[SchematicComponent],
        wires: &[SchematicWire],
    ) {
        // 1. Top Control Bar: Synthesis Actions, Technology Mapping Toggles & Camera Presets
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 4.0);

            let synth_btn = ui.button(
                RichText::new("Synthesize 3D Card")
                    .strong()
                    .color(Color32::from_rgb(100, 220, 255)),
            );
            if synth_btn.clicked() {
                if let Err(e) = self.run_synthesis(components, wires) {
                    self.last_synthesis_error = Some(e.to_string());
                }
            }

            ui.separator();

            ui.checkbox(
                &mut self.tech_options.auto_explode_gates,
                "Explode Abstract Gates to Physical ICs",
            );
            ui.checkbox(
                &mut self.tech_options.force_universal_nand_explosion,
                "Decompose to Pure NAND (74HC00)",
            );

            ui.separator();

            ui.label("Camera:");
            if ui.button("Perspective").clicked() {
                self.reset_camera_perspective();
            }
            if ui.button("Top").clicked() {
                self.reset_camera_top();
            }
            if ui.button("Isometric").clicked() {
                self.reset_camera_isometric();
            }
            if ui.button("Reset").clicked() {
                self.reset_camera_perspective();
            }

            ui.separator();

            ui.checkbox(&mut self.show_abstracted_ground, "Abstracted Ground");
            ui.checkbox(&mut self.show_copper_traces, "Copper Traces");
        });

        // 2. Error Diagnostics Banner
        if let Some(err) = &self.last_synthesis_error {
            ui.add_space(4.0);
            egui::Frame::new()
                .fill(Color32::from_rgb(55, 18, 20))
                .stroke(Stroke::new(1.0, Color32::from_rgb(220, 70, 75)))
                .inner_margin(egui::Margin::symmetric(10, 6))
                .corner_radius(4.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("Synthesis Warning: {}", err))
                                .color(Color32::from_rgb(255, 180, 185))
                                .size(11.5),
                        );
                    });
                });
        }

        // 3. Telemetry Header (if card is synthesized)
        if let Some(card) = &self.synthesized_card {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "Card: {:.1}x{:.1} mm | Chips: {} | Nets: {} | Max EMI Crosstalk: {:.1} mV | Ground Bounce: {:.1} mV",
                        card.board_width_mm,
                        card.board_height_mm,
                        card.chips.len(),
                        card.routed_nets.len(),
                        card.wiring_report.max_crosstalk_noise_mv,
                        card.wiring_report.estimated_ground_bounce_mv,
                    ))
                    .size(11.0)
                    .color(Color32::from_rgb(170, 195, 225)),
                );
            });
        }

        ui.add_space(4.0);

        // 4. Interactive 3D Canvas
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), ui.available_height().max(250.0)),
            Sense::click_and_drag(),
        );

        // Handle camera orbit & pan input
        if response.dragged() {
            let delta = response.drag_delta();
            if ui.input(|i| i.pointer.button_down(egui::PointerButton::Secondary))
                || ui.input(|i| i.pointer.button_down(egui::PointerButton::Middle))
            {
                // Pan
                self.camera.pan += delta;
            } else {
                // Orbit
                self.camera.yaw_deg = (self.camera.yaw_deg + delta.x * 0.45) % 360.0;
                self.camera.pitch_deg = (self.camera.pitch_deg - delta.y * 0.45).clamp(5.0, 89.9);
            }
        }

        // Handle scroll zoom
        let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll_delta.abs() > 0.0 {
            self.camera.zoom = (self.camera.zoom + scroll_delta * 0.015).clamp(2.0, 40.0);
        }

        let painter = ui.painter_at(rect);

        // Fill background
        painter.rect_filled(rect, 0.0, Color32::from_rgb(12, 14, 18));

        let center_screen = rect.center() + self.camera.pan;
        let scale = self.camera.zoom;
        let yaw_rad = self.camera.yaw_deg.to_radians();
        let pitch_rad = self.camera.pitch_deg.to_radians();

        let cos_yaw = yaw_rad.cos();
        let sin_yaw = yaw_rad.sin();
        let cos_pitch = pitch_rad.cos();
        let sin_pitch = pitch_rad.sin();

        // 3D Projection closure
        let project_pt = |vx: f32, vy: f32, vz: f32| -> Pos2 {
            // Yaw rotation around Z
            let x1 = vx * cos_yaw - vy * sin_yaw;
            let y1 = vx * sin_yaw + vy * cos_yaw;

            // Pitch rotation around X
            let y2 = y1 * cos_pitch - vz * sin_pitch;
            let z2 = y1 * sin_pitch + vz * cos_pitch;

            // Perspective factor
            let focal = 400.0;
            let camera_dist = 220.0;
            let factor = focal / (focal + y2 + camera_dist);

            let sx = center_screen.x + (x1 * factor * scale);
            let sy = center_screen.y - (z2 * factor * scale);
            Pos2::new(sx, sy)
        };

        // 5. Draw Abstracted Ground Plane & Grid Lines
        if self.show_abstracted_ground {
            let ground_z = -8.0;
            let grid_size = 120.0;
            let step = 15.0;

            // Ground grid lines
            let grid_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(35, 55, 85, 90));
            let mut x = -grid_size;
            while x <= grid_size {
                let p1 = project_pt(x, -grid_size, ground_z);
                let p2 = project_pt(x, grid_size, ground_z);
                painter.line_segment([p1, p2], grid_stroke);
                x += step;
            }
            let mut y = -grid_size;
            while y <= grid_size {
                let p1 = project_pt(-grid_size, y, ground_z);
                let p2 = project_pt(grid_size, y, ground_z);
                painter.line_segment([p1, p2], grid_stroke);
                y += step;
            }
        }

        // Auto-synthesize default baseline if empty
        if self.synthesized_card.is_none() && !components.is_empty() {
            let _ = self.run_synthesis(components, wires);
        }

        if let Some(card) = &self.synthesized_card {
            // 6. Draw Card Drop Shadow on Ground Plane
            if self.show_shadow {
                let ground_z = -7.9;
                let hw = card.board_width_mm / 2.0;
                let hh = card.board_height_mm / 2.0;
                let s_pts = [
                    project_pt(-hw + 2.0, -hh - 2.0, ground_z),
                    project_pt(hw + 4.0, -hh - 2.0, ground_z),
                    project_pt(hw + 4.0, hh + 2.0, ground_z),
                    project_pt(-hw + 2.0, hh + 2.0, ground_z),
                ];
                painter.add(egui::Shape::convex_polygon(
                    s_pts.to_vec(),
                    Color32::from_rgba_unmultiplied(10, 12, 16, 140),
                    Stroke::NONE,
                ));
            }

            // 7. Render 3D Faces of the Card with Depth-Sorting (Painter's Algorithm)
            let mut faces = card.generate_3d_faces();

            // Compute depth distance from camera for each face
            let compute_face_depth = |f: &Face3D| -> f32 {
                let vx = f.center.x;
                let vy = f.center.y;
                let vz = f.center.z;
                let y1 = vx * sin_yaw + vy * cos_yaw;
                let y2 = y1 * cos_pitch - vz * sin_pitch;
                y2
            };

            // Sort back to front (largest depth distance first)
            faces.sort_by(|a, b| {
                let da = compute_face_depth(a);
                let db = compute_face_depth(b);
                db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
            });

            // Light source vector in world space: (0.4, -0.4, 0.8) normalized
            let lx = 0.408;
            let ly = -0.408;
            let lz = 0.816;

            for face in faces {
                // Apply simple Lambertian directional lighting to face color
                let dot = (face.normal.x * lx + face.normal.y * ly + face.normal.z * lz).max(0.0);
                let light_factor = 0.65 + 0.35 * dot;

                let lit_color = Color32::from_rgb(
                    ((face.color.r() as f32) * light_factor).clamp(0.0, 255.0) as u8,
                    ((face.color.g() as f32) * light_factor).clamp(0.0, 255.0) as u8,
                    ((face.color.b() as f32) * light_factor).clamp(0.0, 255.0) as u8,
                );

                let screen_pts: Vec<Pos2> = face
                    .vertices
                    .iter()
                    .map(|v| project_pt(v.x, v.y, v.z))
                    .collect();

                if screen_pts.len() >= 3 {
                    painter.add(egui::Shape::convex_polygon(
                        screen_pts,
                        lit_color,
                        Stroke::new(0.5, Color32::from_rgb(20, 22, 26)),
                    ));
                }
            }

            // 8. Render Routed Copper Traces on Top Surface (Z = 0.05)
            if self.show_copper_traces {
                let hw = card.board_width_mm / 2.0;
                let hh = card.board_height_mm / 2.0;

                for net in &card.routed_nets {
                    for seg in &net.segments {
                        let z = if seg.layer == CopperLayer::Top {
                            0.05
                        } else {
                            -card.board_thickness_mm - 0.05
                        };
                        let p1 = project_pt(seg.start_mm.x - hw, seg.start_mm.y - hh, z);
                        let p2 = project_pt(seg.end_mm.x - hw, seg.end_mm.y - hh, z);

                        let trace_color = if seg.layer == CopperLayer::Top {
                            Color32::from_rgb(215, 110, 45) // Warm copper / top
                        } else {
                            Color32::from_rgb(45, 160, 215) // Blue copper / bottom
                        };

                        painter.line_segment(
                            [p1, p2],
                            Stroke::new(seg.width_mm * scale * 0.45 + 1.2, trace_color),
                        );
                    }
                }
            }

            // 9. Silkscreen Labels for Chips
            let hw = card.board_width_mm / 2.0;
            let hh = card.board_height_mm / 2.0;
            for chip in &card.chips {
                let cz = chip.package_type.height_mm() + 0.05;
                let label_pos = project_pt(chip.center_mm.x - hw, chip.center_mm.y - hh, cz);
                painter.text(
                    label_pos,
                    egui::Align2::CENTER_CENTER,
                    &chip.chip_part_number,
                    egui::FontId::monospace(10.0 * (scale * 0.12).clamp(0.7, 1.4)),
                    Color32::from_rgb(240, 245, 250),
                );
            }
        } else {
            // Prompt to synthesize
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Click 'Synthesize 3D Card' above to synthesize a physical board card from the schematic.",
                egui::FontId::proportional(14.0),
                Color32::from_rgb(140, 155, 175),
            );
        }
    }
}
